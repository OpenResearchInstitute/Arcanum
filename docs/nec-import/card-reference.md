# NEC Import — Card Reference

**Project:** Arcanum  
**Document:** `docs/nec-import/card-reference.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document is the field-by-field reference for every NEC card type supported by Arcanum's import parser. It is the authoritative source for how NEC card fields map to Arcanum's internal data structures. 

This document is derived from the NEC-2 User's Manual (Burke & Poggio, 1981). Where NEC-2 behavior is ambiguous or where Arcanum intentionally diverges, that is noted explicitly. If you find a difference that is not in here, please file an issue.

---

## 2. NEC Card Format

### 2.1 General Structure

Every line in a `.nec` file begins with a two-character card mnemonic followed by data fields. Two field types exist:

- **Integer fields** — whole numbers, no decimal point
- **Float fields** — real numbers, may use decimal point or scientific notation

The traditional NEC-2 format is column-based (inherited from punched cards). Modern tools including Arcanum accept free-field (whitespace-delimited) format. Arcanum must handle both.

### 2.2 Column-Based Format

In strict column format, integer fields occupy columns 3–5 and 6–10, and float fields occupy ten-character columns thereafter. Arcanum's parser must not require strict column alignment but must accept it.

### 2.3 Comments

Lines beginning with `CM` are comment lines and are ignored by the parser. A `CE` card ends the comment block. Neither affects the mesh or simulation parameters.

### 2.4 Card Ordering

Geometry cards (`GW`, `GA`, `GH`, `GM`, `GS`) must appear before `GE`. All other cards (`EX`, `LD`, `FR`, `GN`, `RP`, `NE`, `NH`) appear after `GE`. `EN` terminates the deck. `GE` and `EN` are required.

---

## 3. Geometry Cards

### GW — Straight Wire

Creates one or more straight wire segments between two endpoints.

**Phase routing:** Phase 1 (Geometry)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | ITAG | Wire tag number. Must be > 0. Used by EX and LD cards to reference this wire. |
| 2 | Integer | NS | Number of segments. Must be ≥ 1. |
| 3 | Float | XW1 | X coordinate of end 1 (meters) |
| 4 | Float | YW1 | Y coordinate of end 1 (meters) |
| 5 | Float | ZW1 | Z coordinate of end 1 (meters) |
| 6 | Float | XW2 | X coordinate of end 2 (meters) |
| 7 | Float | YW2 | Y coordinate of end 2 (meters) |
| 8 | Float | ZW2 | Z coordinate of end 2 (meters) |
| 9 | Float | RAD | Wire radius (meters). Must be > 0. |

**Parametric mapping:** See `docs/phase1-geometry/math.md` Section 3. End 1 maps to r_a, end 2 maps to r_b.

**Hard errors:** NS = 0; zero-length wire (end 1 = end 2); RAD ≤ 0.

**Notes:**
- ITAG must be unique within the deck. Duplicate tags produce a hard error.
- Segments are numbered 1 through NS along the wire from end 1 to end 2. This numbering is used by EX and LD cards.

---

### GA — Circular Arc

Creates a circular arc in the **XZ plane**, centered at the origin. Angles are measured from the positive x-axis toward the positive z-axis.

**Phase routing:** Phase 1 (Geometry)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | ITAG | Wire tag number. Must be > 0. |
| 2 | Integer | NS | Number of segments. Must be ≥ 1. |
| 3 | Float | RADA | Arc radius (meters). Must be > 0. |
| 4 | Float | ANG1 | Start angle (degrees) |
| 5 | Float | ANG2 | End angle (degrees) |
| 6 | Float | RAD | Wire radius (meters). Must be > 0. |

**Parametric mapping:** See `docs/phase1-geometry/math.md` Section 4. RADA maps to R, ANG1 and ANG2 map to θ₁ and θ₂ (converted to radians).

**Hard errors:** NS = 0; RADA ≤ 0; RAD ≤ 0; ANG1 = ANG2 (zero-length arc).

**Notes:**
- ANG2 > ANG1 for a counterclockwise arc. ANG2 < ANG1 is valid and produces a clockwise arc.
- ANG2 - ANG1 = 360° produces a full circle (loop antenna). The parser must detect this and set the self-loop flag on the resulting junction.
- The arc lies in the XZ plane. To rotate to an arbitrary plane, use GM after GA.

---

### GH — Helix

Creates a helix with its axis along the z-axis, starting at the origin.

**Phase routing:** Phase 1 (Geometry)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | ITAG | Wire tag number. Must be > 0. |
| 2 | Integer | NS | Number of segments. Must be ≥ 1. |
| 3 | Float | S | Turn spacing / pitch (meters per turn). Axial advance per full rotation. |
| 4 | Float | HL | Total axial length of helix (meters). Must be > 0. |
| 5 | Float | A1 | Helix radius at start — z = 0 end (meters). Must be > 0. |
| 6 | Float | A2 | Helix radius at end — z = HL end (meters). Must be > 0. |
| 7 | Float | RAD | Wire radius (meters). Must be > 0. |

**Parametric mapping:** See `docs/phase1-geometry/math.md` Section 5.

```
N_turns = HL / S
A(τ)    = A1 + τ(A2 - A1)      (uniform helix when A1 = A2)
```

**Hard errors:** NS = 0; S = 0; HL ≤ 0; A1 ≤ 0; A2 ≤ 0; RAD ≤ 0.

**Notes:**
- S > 0 produces a right-handed helix advancing in +z. S < 0 is not standard NEC-2 but Arcanum must handle it gracefully (left-handed helix advancing in -z) rather than silently producing wrong geometry.
- A1 = A2 is the common case (uniform helix). A1 ≠ A2 produces a tapered helix. The arc length element is no longer constant along the helix. See `math.md` Section 5.4.
- N_turns = HL / S need not be an integer. Non-integer turns are valid and produce a helix that does not complete a full final turn.

---

### GM — Geometry Move / Rotate / Scale

Applies a rotation, translation, and optional replication to a set of wires identified by tag number.

**Phase routing:** Phase 1 (Geometry). Applied eagerly. The mesh stores final coordinates only.

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | ITAG | Tag number of wire to transform. 0 = all wires. |
| 2 | Integer | NRPT | Number of additional copies to generate. 0 = transform in place (no copy). |
| 3 | Float | ROX | Rotation about x-axis (degrees) |
| 4 | Float | ROY | Rotation about y-axis (degrees) |
| 5 | Float | ROZ | Rotation about z-axis (degrees) |
| 6 | Float | XS | Translation along x-axis (meters) |
| 7 | Float | YS | Translation along y-axis (meters) |
| 8 | Float | ZS | Translation along z-axis (meters) |
| 9 | Integer | ITS | Tag number increment for generated copies. |

**Transformation order:** Rotation is applied first (about the origin), then translation.

**Rotation order:** ROX, then ROY, then ROZ (intrinsic rotations about the original axes in that sequence).

**Replication:** When NRPT > 0, NRPT additional copies of the specified wire(s) are generated, each copy offset by one additional application of the transform relative to the previous. Tags of copies are incremented by ITS per copy. The original wire is not moved. Only copies are generated.

**Hard errors:** ITS = 0 when NRPT > 0 (would produce duplicate tags).

**Notes:**
- GM transforms coordinates of wire endpoints, not the parametric forms. The parametric form is re-derived from the transformed endpoints.
- Wire radii are not affected by rotation or translation. They are not affected by scale (GS handles scaling; see note in GS).
- Multiple GM cards are applied in the order they appear in the deck.

---

### GS — Global Scale

Scales all wire coordinates by a single factor. Must appear before GE.

**Phase routing:** Phase 1 (Geometry) — applied before all GM transformations.

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | — | Not used. Set to 0. |
| 2 | Integer | — | Not used. Set to 0. |
| 3 | Float | XSCALE | Scale factor applied to all wire coordinates. |

**Hard errors:** XSCALE = 0.

**Critical note:** GS scales wire endpoint coordinates only. **Wire radii are not scaled.** This is NEC-2 standard behavior and is intentional. A model built in wavelengths is scaled to meters at a given frequency, but the wire radius is specified directly in the physical unit. This behavior is explicitly validated in `validation.md` case V-TRF-001.

---

### GE — Geometry End

Terminates the geometry section. Required. All geometry cards must appear before GE. All other cards must appear after GE.

**Phase routing:** Phase 1 (parser control)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | GPFLAG | Ground plane flag |

**GPFLAG values:**

| Value | Meaning |
|---|---|
| 0 | No ground plane present |
| 1 | Ground plane present. Wire segments whose lower end touches z = 0 are treated as monopoles over a ground plane. The segment is not extended below ground. |
| -1 | Ground plane present. Segments touching z = 0 are not treated specially (less common). |

**Hard error:** Missing GE card — the parser must abort with a descriptive error.

**Notes:**
- GPFLAG = 1 is the standard value when a GN card specifying a ground plane is present.
- The ground plane geometry itself is specified by GN, not GE. GE's GPFLAG controls only the monopole treatment at z = 0.

---

## 4. Environment Cards

### GN — Ground Definition

Specifies the ground plane type and electrical parameters. If absent, free space is assumed.

**Phase routing:** Dual — Phase 1 (geometric boundary condition); Phase 2 (electrical parameters). See `docs/phase1-geometry/design.md` Section 6.1 for the split rationale.

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | IPERF | Ground type (see table below) |
| 2 | Integer | NRADL | Number of radial wires in ground screen. 0 if no ground screen. |
| 3 | Float | — | Not used. Set to 0. |
| 4 | Float | — | Not used. Set to 0. |
| 5 | Float | EPSE | Relative dielectric constant (permittivity) of ground |
| 6 | Float | SIG | Conductivity of ground (S/m) |

**IPERF values:**

| Value | Meaning | Phase 1 Action | Phase 2 Action |
|---|---|---|---|
| -1 | Free space (no ground) | No images generated | No ground correction |
| 0 | Finite ground, reflection coefficient approximation | No images generated | Use EPSE, SIG |
| 1 | Perfect ground (PEC) | Generate image segments | No Sommerfeld integral needed |
| 2 | Finite ground, Sommerfeld/Wait theory | No images generated | Use EPSE, SIG with Wait model |

**Hard errors:** EPSE ≤ 0 when IPERF = 0 or 2; SIG < 0.

**Notes:**
- NRADL > 0 (radial wire ground screen) is a specialized broadcast antenna feature. Arcanum initial implementation may defer this. Flag as a warning if NRADL > 0.
- Only one GN card per deck. Multiple GN cards produce a hard error.
- When IPERF = 1 (PEC), EPSE and SIG are present in the card but are ignored by both phases.

---

## 5. Excitation and Load Cards

### EX — Excitation

Applies a source to a specific segment. Phase 3 consumes this card to assemble the excitation vector [V].

**Phase routing:** Phase 3 (Matrix Solve)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | EXTYPE | Excitation type (see table below) |
| 2 | Integer | ITAG | Tag number of wire containing the source segment |
| 3 | Integer | ISEG | Segment number on the wire (1-indexed from end 1) |
| 4 | Integer | NEXP | Not used for type 0. Set to 0. |
| 5 | Float | EXREAL | Real part of applied voltage (volts) |
| 6 | Float | EXIMAG | Imaginary part of applied voltage (volts) |

**EXTYPE values supported in initial implementation:**

| Value | Meaning |
|---|---|
| 0 | Voltage source (delta-gap model) — standard driven element |
| 5 | Voltage source (current slope discontinuity model) |

Types 1–4 (incident plane wave, elementary current source) are deferred. Arcanum must emit a warning if EXTYPE ∈ {1, 2, 3, 4} and skip the card.

**Hard errors:** ITAG references a tag not present in the mesh; ISEG > NS for that wire; EXTYPE not in {0, 5} (warning, not error, per above).

---

### LD — Load

Places an impedance load on one or more segments. Phase 3 consumes this card.

**Phase routing:** Phase 3 (Matrix Solve)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | LDTYPE | Load type (see table below) |
| 2 | Integer | ITAG | Tag number of wire to load. 0 = all wires. |
| 3 | Integer | LDTAGF | First segment number to load (1-indexed) |
| 4 | Integer | LDTAGT | Last segment number to load (1-indexed). If 0, load only LDTAGF. |
| 5 | Float | ZLR | Resistance (Ω) or conductivity (S/m) depending on LDTYPE |
| 6 | Float | ZLI | Inductance (H) or reactance (Ω) depending on LDTYPE |
| 7 | Float | ZLC | Capacitance (F) depending on LDTYPE |

**LDTYPE values:**

| Value | Meaning |
|---|---|
| 0 | Series RLC lumped load |
| 1 | Parallel RLC lumped load |
| 4 | Series RLC distributed (per unit length) |
| 5 | Wire conductivity (S/m) — distributed resistive loss |

**Hard errors:** ITAG references unknown tag; LDTAGF > LDTAGT (when LDTAGT ≠ 0).

---

## 6. Frequency Card

### FR — Frequency

Specifies the frequency or frequency sweep for the simulation. Phase 2 and Phase 3 consume this card.

**Phase routing:** Phase 2 (Matrix Fill), Phase 3 (Matrix Solve)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | IFRQ | Stepping type: 0 = linear, 1 = multiplicative |
| 2 | Integer | NFRQ | Number of frequency steps. Must be ≥ 1. |
| 3 | Integer | — | Not used. Set to 0. |
| 4 | Integer | — | Not used. Set to 0. |
| 5 | Float | FMHZ | Starting frequency (MHz) |
| 6 | Float | DELFRQ | Frequency increment (MHz if IFRQ=0; multiplicative factor if IFRQ=1) |

**Hard errors:** NFRQ < 1; FMHZ ≤ 0; DELFRQ = 0 when NFRQ > 1.

**Notes:**
- A single frequency simulation uses NFRQ = 1 and DELFRQ = 0.
- Multiple FR cards are valid and produce a concatenated frequency list.

---

## 7. Output Request Cards

### RP — Radiation Pattern

Requests a far-field radiation pattern computation. Phase 4 consumes this card.

**Phase routing:** Phase 4 (Post-Processing)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | CALC | Calculation type: 0 = major axis, 1 = normalized |
| 2 | Integer | NTHETA | Number of theta (elevation) angles |
| 3 | Integer | NPHI | Number of phi (azimuth) angles |
| 4 | Integer | XNDA | Output format and normalization flags |
| 5 | Float | THETS | Starting theta angle (degrees, 0 = zenith) |
| 6 | Float | PHIS | Starting phi angle (degrees) |
| 7 | Float | DTHS | Theta increment (degrees) |
| 8 | Float | DPHS | Phi increment (degrees) |
| 9 | Float | RFLD | Radial distance. 0 = far field. |

---

### NE — Near Electric Field

Requests near electric field computation on a rectangular grid. Phase 4 consumes this card.

**Phase routing:** Phase 4 (Post-Processing)

| Field | Type | Name | Description |
|---|---|---|---|
| 1 | Integer | — | Not used |
| 2 | Integer | NX | Number of x points |
| 3 | Integer | NY | Number of y points |
| 4 | Integer | NZ | Number of z points |
| 5 | Float | XO | X origin of grid (meters) |
| 6 | Float | YO | Y origin of grid (meters) |
| 7 | Float | ZO | Z origin of grid (meters) |
| 8 | Float | DX | X spacing (meters) |
| 9 | Float | DY | Y spacing (meters) |
| 10 | Float | DZ | Z spacing (meters) |

---

### NH — Near Magnetic Field

Identical field layout to NE. Requests near magnetic field computation. Phase 4 consumes this card.

**Phase routing:** Phase 4 (Post-Processing)

Fields identical to NE.

---

## 8. Deck Control Cards

### EN — End of Input Deck

Terminates the input deck. Required. No fields.

**Hard error:** Missing EN card — parser must emit a warning (not a hard error, as many real-world decks omit EN and are otherwise valid).

---

### CM / CE — Comments

`CM` lines are comment lines and are ignored. `CE` ends the comment block. Both may appear anywhere before `GE`.

---

## 9. Unsupported Cards

The following NEC-2 cards are not supported in the initial Arcanum implementation. The parser must emit a warning and skip them without aborting.

| Card | Description | Deferral Reason |
|---|---|---|
| GR | Geometry rotate (generate copies by rotation) | Superseded by GM for initial scope |
| SP | Surface patch | Surface patch MoM not in scope |
| SM | Surface patch mesh | Surface patch MoM not in scope |
| NT | Network (two-port) | Deferred |
| TL | Transmission line | Deferred |
| KH | Interaction approximation range | Solver parameter, deferred |
| NX | Next frequency (deprecated) | Deprecated in NEC-2 |
| PQ | Print control | Output formatting, deferred |
| PT | Print control | Output formatting, deferred |
| XQ | Execute (deprecated) | Deprecated |

---

## 10. References

- Burke, G.J. & Poggio, A.J. — *Numerical Electromagnetics Code (NEC-2) Method of Moments*, Lawrence Livermore National Laboratory, 1981 — authoritative card definitions
- `docs/nec-import/design.md` — parser architecture and card routing
- `docs/nec-import/validation.md` — test cases referencing these field definitions
- `docs/phase1-geometry/math.md` — parametric mappings for GW, GA, GH fields

---

*Arcanum — Open Research Institute*
