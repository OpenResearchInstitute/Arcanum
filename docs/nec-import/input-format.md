# NEC Import — Input Format Guide

**Project:** Arcanum  
**Document:** `docs/nec-import/input-format.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document is the practical guide to using Arcanum's NEC deck parser. It covers:

- What a `.nec` file looks like and what sections it contains
- How to call the parser from Python
- What each error and warning means and how to resolve it
- Which cards are supported and which are deferred

For field-by-field card definitions, see [`card-reference.md`](card-reference.md).  
For test cases and validation, see [`validation.md`](validation.md).

---

## 2. The NEC Deck Format

A `.nec` file is a complete simulation input deck. It specifies geometry, excitation, loads, ground model, frequency, and output requests in a card-based text format inherited from punched-card computing. Each line begins with a two-character card mnemonic followed by numeric fields.

### 2.1 Minimal Deck Structure

```
CM  Optional comment lines
CM  Additional comment (ignored by parser)
CE  End of comment block

GW 1 10  0.0 0.0 -0.25  0.0 0.0 0.25  0.002
GE 0

FR 0 1 0 0  146.0  0.0
EX 0 1 5 0  1.0 0.0

EN
```

The deck has two sections separated by the `GE` card:

1. **Geometry section** — `GW`, `GA`, `GH`, `GM`, `GS` cards, terminated by `GE`
2. **Control section** — `EX`, `LD`, `FR`, `GN`, `RP`, `NE`, `NH` cards, terminated by `EN`

Both `GE` and `EN` are required. The parser will return an error if either is missing.

### 2.2 Field Format

Fields are whitespace-delimited. Traditional column-aligned format (inherited from punched cards) is also accepted. Each card has two integer fields followed by up to ten float fields:

```
GW  1   10   0.0  0.0  -0.25   0.0  0.0  0.25   0.002
    ^    ^    ^                                    ^
    ITAG NS   XW1 YW1   ZW1    XW2  YW2  ZW2    radius
```

Floats may use scientific notation (`1.5e-3`). Integer fields normally contain plain integers (`10`), but some NEC generators (e.g. 4nec2) write every field in scientific notation — including integers — producing values such as `0.00000E+00` for zero. The parser accepts integer fields in scientific notation provided the value is a whole number.

### 2.3 Comments

`CM` lines are ignored by the parser. A `CE` card ends the comment block. Comment cards may appear anywhere before `GE`.

---

## 3. Python API

The parser is exposed via the `arcanum` Python module (compiled from the `arcanum-py` crate).

### 3.1 Parsing a String

```python
import arcanum

deck = """
GW 1 10  0.0 0.0 -0.25  0.0 0.0 0.25  0.002
GE 0
FR 0 1 0 0  146.0  0.0
EX 0 1 5 0  1.0 0.0
EN
"""

sim, warnings = arcanum.nec_import.parse(deck)

print(sim.mesh_input.wires)          # list of StraightWire / ArcWire / HelixWire
print(sim.output_requests.radiation_patterns)  # list of RadiationPatternRequest
for w in warnings:
    print(w.kind, w.message)
```

### 3.2 Parsing a File

```python
sim, warnings = arcanum.nec_import.parse_file("my_antenna.nec")
```

### 3.3 Handling Parse Errors

`parse()` and `parse_file()` raise `arcanum.ParseError` on failure. The exception carries three attributes:

| Attribute | Type | Description |
|---|---|---|
| `kind` | `str` | Machine-readable error category (see Section 5) |
| `line` | `int` | 1-indexed line number in the input where the error was detected |
| `message` | `str` | Human-readable description of the problem |

```python
try:
    sim, warnings = arcanum.nec_import.parse(deck)
except arcanum.ParseError as e:
    print(f"Parse failed at line {e.line}: {e.message}")
    print(f"Error kind: {e.kind}")
```

### 3.4 Inspecting Simulation Input

The `SimulationInput` object returned by `parse()` has the following structure:

```
SimulationInput
├── mesh_input: MeshInput
│   ├── wires: list[StraightWire | ArcWire | HelixWire]
│   └── ground: GeometricGround | None
├── sources: list[SourceDefinition]
├── loads: list[LoadDefinition]
└── output_requests: OutputRequests
    ├── radiation_patterns: list[RadiationPatternRequest]
    ├── near_electric_fields: list[NearFieldRequest]
    └── near_magnetic_fields: list[NearFieldRequest]
```

**Wire types** — the `wires` list is heterogeneous; check the type of each element:

```python
for wire in sim.mesh_input.wires:
    if isinstance(wire, arcanum.StraightWire):
        print(f"tag={wire.tag}  segments={wire.num_segments}  radius={wire.radius}")
    elif isinstance(wire, arcanum.ArcWire):
        print(f"tag={wire.tag}  arc_radius={wire.arc_radius}  ang1={wire.ang1_deg}  ang2={wire.ang2_deg}")
    elif isinstance(wire, arcanum.HelixWire):
        print(f"tag={wire.tag}  turns={wire.total_length / wire.turn_spacing}")
```

**Ground** — `mesh_input.ground` is `None` if no `GN` card is present (free space). If present, it is a `GeometricGround` with a `ground_type` string (`"FreeSpace"`, `"PEC"`, `"Lossy"`, `"Sommerfeld"`) and optional `electrical` parameters (relative permittivity and conductivity).

---

## 4. Supported Cards

### Geometry Section

| Card | Support | Description |
|---|---|---|
| `GW` | Full | Straight wire |
| `GA` | Full | Circular arc |
| `GH` | Full | Helix |
| `GM` | Full | Geometry move / rotate / replicate |
| `GS` | Full | Global scale |
| `GE` | Required | Geometry end (required terminator) |
| `CM` / `CE` | Full | Comments (ignored) |

### Control Section

| Card | Support | Description |
|---|---|---|
| `GN` | Full | Ground definition |
| `EX` | Full | Excitation (voltage source) |
| `LD` | Full | Load (impedance) |
| `FR` | Full | Frequency |
| `RP` | Full | Radiation pattern request |
| `NE` | Full | Near electric field request |
| `NH` | Full | Near magnetic field request |
| `EN` | Required | End of deck (required terminator) |

### Deferred Cards (produce a warning, then skipped)

| Card | Description |
|---|---|
| `GR` | Geometry rotate (generate copies by rotation) |
| `SP` / `SM` | Surface patch — not in scope |
| `NT` | Network (two-port) |
| `TL` | Transmission line |
| `KH` | Interaction approximation range |
| `NX` / `XQ` | Deprecated NEC-2 cards |
| `PQ` / `PT` | Print control (output formatting) |

Any unrecognized card mnemonic also produces a warning and is skipped.

---

## 5. Error Reference

`ParseError.kind` is one of the following strings:

| Kind | Meaning |
|---|---|
| `MissingGeEnd` | `GE` card is absent — geometry section was never closed |
| `MissingEnd` | `EN` card is absent — deck was not properly terminated |
| `DuplicateTag` | Two geometry cards use the same tag number |
| `UnknownTag` | An `EX` or `LD` card references a tag that does not appear in the mesh |
| `FieldParseFailure` | A field could not be parsed (wrong type, missing, extra fields) |
| `InvalidField` | A field was parsed but its value is out of range (e.g. `NS = 0`, `RAD ≤ 0`) |
| `DuplicateGN` | More than one `GN` card appears in the deck |
| `DuplicateFR` | More than one `FR` card appears in the deck |

### Resolving Common Errors

**`MissingGeEnd`** — Ensure your deck contains `GE 0` (or `GE 1`) after the last geometry card and before any control cards.

**`DuplicateTag`** — Every `GW`, `GA`, and `GH` card must have a unique ITAG (first field). If you use `GM` with NRPT > 0, ensure the ITS (tag increment) field is set so generated copies do not collide with existing tags.

**`UnknownTag`** — The tag number in an `EX` or `LD` card does not match any geometry card. Check for typos in the tag field.

**`FieldParseFailure`** — The number of fields on a card does not match expectations, or an integer field contains a decimal point, or a float field contains non-numeric text. The `line` attribute identifies the offending card.

**`InvalidField`** — A field is present and correctly typed but out of range. Common causes:
- `NS = 0` on a `GW`, `GA`, or `GH` card (must be ≥ 1)
- Wire radius ≤ 0
- `ANG1 = ANG2` on a `GA` card (zero-length arc)
- `S = 0` or `HL ≤ 0` on a `GH` card

---

## 6. Warning Reference

Warnings do not abort parsing. They are returned as the second element of the tuple from `parse()` / `parse_file()`. Each warning has `kind`, `line`, and `message` attributes.

| Kind | Meaning |
|---|---|
| `UnsupportedCard` | A recognized but deferred card (e.g. `GR`, `TL`) was skipped |
| `UnknownCard` | An unrecognized card mnemonic was skipped |
| `UnsupportedExType` | An `EX` card with an unsupported `EXTYPE` value (not 0 or 5) was skipped |
| `RadialGroundScreen` | A `GN` card with `NRADL > 0` was present; radial ground screen support is deferred |
| `MissingEnd` | The `EN` card was absent (this is a warning, not an error, since many real-world decks omit it) |

### When to Act on Warnings

- **`UnsupportedCard` / `UnknownCard`**: If the skipped card was critical to your simulation (e.g. a transmission line `TL`), the simulation results will be incomplete. Verify whether the card's effect can be approximated with supported cards.
- **`UnsupportedExType`**: The excitation was not applied. The simulation will run but with no driven element.
- **`RadialGroundScreen`**: The ground model will use the electrical parameters from `GN` but the ground screen effect is not modeled.

---

## 7. Example Decks

For complete, validated reference decks, see [`reference-decks/`](reference-decks/). These files are used by the V-REAL integration test cases:

| File | Description |
|---|---|
| `half-wave-dipole.nec` | Center-fed half-wave dipole at 146 MHz, 10 segments |
| `yagi-3el.nec` | 3-element Yagi-Uda at 146 MHz (reflector, driven element, director) |
| `helix-axial.nec` | Axial-mode helix antenna for 435 MHz |
| `dumbbell-ori.nec` | ORI dumbbell reference geometry |

---

## 8. References

- [`card-reference.md`](card-reference.md) — field-by-field card definitions
- [`validation.md`](validation.md) — test case specifications
- Burke, G.J. & Poggio, A.J. — *Numerical Electromagnetics Code (NEC-2) Method of Moments*, LLNL, 1981

---

*Arcanum — Open Research Institute*
