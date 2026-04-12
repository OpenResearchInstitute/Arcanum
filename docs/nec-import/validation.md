# NEC Import — Validation

**Project:** Arcanum  
**Document:** `docs/nec-import/validation.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the validation cases for the NEC import parser. The parser's responsibilities are:

1. Read a `.nec` input deck (file or string)
2. Parse each card's fields into typed internal structures
3. Route each card's data to the correct phase's input structures
4. Emit hard errors on invalid input that must abort parsing
5. Emit warnings on suspicious input that should be reported but not abort

**Scope boundary:** This document tests the parser layer only. Whether the resulting `Mesh` has geometrically correct segment endpoints is the concern of `docs/phase1-geometry/validation.md`. Whether the impedance matrix is correctly filled is the concern of Phase 2 validation. The parser is correct when it correctly reads fields and routes them. Not when the downstream physics is correct.

---

## 2. Test Case Naming

Cases are named by category:

- `V-PARSE-XXX` — correct field parsing for each card type
- `V-FMT-XXX` — input format handling (column vs free-field)
- `V-ROUTE-XXX` — card routing to correct phase data structures
- `V-ERR-XXX` — hard error cases
- `V-WARN-XXX` — warning cases
- `V-REAL-XXX` — complete real-world reference decks

---

## 3. Field Parsing Cases (V-PARSE)

### V-PARSE-001 — GW Card Field Parsing

**Input:**
```
GW 3 10 0.0 0.0 -0.25 0.0 0.0 0.25 0.002
GE 0
EN
```

**Expected parsed fields:**
- ITAG: 3 (integer)
- NS: 10 (integer)
- XW1: 0.0, YW1: 0.0, ZW1: -0.25 (floats, meters)
- XW2: 0.0, YW2: 0.0, ZW2: 0.25 (floats, meters)
- RAD: 0.002 (float, meters)

**Pass criterion:** All fields parsed to correct types and values. No warnings. No errors.

---

### V-PARSE-002 — GA Card Field Parsing

**Input:**
```
GA 2 8 0.15 0.0 360.0 0.001
GE 0
EN
```

**Expected parsed fields:**
- ITAG: 2 (integer)
- NS: 8 (integer)
- RADA: 0.15 (float, meters)
- ANG1: 0.0 (float, degrees)
- ANG2: 360.0 (float, degrees)
- RAD: 0.001 (float, meters)

**Pass criterion:** ANG2 - ANG1 = 360.0 means self-loop flag set on resulting junction.

---

### V-PARSE-003 — GH Card Field Parsing

**Input:**
```
GH 1 16 0.0238 0.119 0.0239 0.0239 0.001
GE 0
EN
```

**Expected parsed fields:**
- ITAG: 1 (integer)
- NS: 16 (integer)
- S: 0.0238 (float, meters — pitch)
- HL: 0.119 (float, meters — total axial length)
- A1: 0.0239 (float, meters — start radius)
- A2: 0.0239 (float, meters — end radius, uniform helix since A1 = A2)
- RAD: 0.001 (float, meters — wire radius)

**Derived check:**
- N_turns = HL / S = 0.119 / 0.0238 = 5.0 (exactly 5 turns, uniform helix)

**Pass criterion:** All fields parsed correctly. N_turns computed as 5.0.

---

### V-PARSE-004 — GN Card, PEC Ground

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 1
GE 1
EN
```

**Expected parsed fields:**
- IPERF: 1 (PEC ground)
- NRADL: 0
- EPSE: not present (or 0.0 — ignored for PEC)
- SIG: not present (or 0.0 — ignored for PEC)

**Phase routing check:**
- Phase 1 receives: GroundType::PEC → generates image segments
- Phase 2 receives: no lossy ground parameters

**Pass criterion:** GroundDescriptor.ground_type = PEC; images_generated = true after Phase 1.

---

### V-PARSE-005 — GN Card, Lossy Ground

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 2 0 0 0 13.0 0.005
GE 1
EN
```

**Expected parsed fields:**
- IPERF: 2 (Sommerfeld/Wait lossy ground)
- NRADL: 0
- EPSE: 13.0
- SIG: 0.005 S/m

**Phase routing check:**
- Phase 1 receives: GroundType::Lossy means that no image segments generated
- Phase 2 receives: EPSE = 13.0, SIG = 0.005

**Pass criterion:** GroundDescriptor.conductivity = 0.005; GroundDescriptor.permittivity = 13.0; images_generated = false.

---

### V-PARSE-006 — EX Card, Voltage Source

**Input:**
```
GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EX 0 1 6 0 1.0 0.0
EN
```

**Expected parsed fields:**
- EXTYPE: 0 (delta-gap voltage source)
- ITAG: 1
- ISEG: 6 (center segment of 11-segment dipole)
- EXREAL: 1.0 V
- EXIMAG: 0.0 V

**Pass criterion:** Source correctly identified as tag 1, segment 6, 1V real excitation.

---

### V-PARSE-007 — FR Card, Single Frequency

**Input:**
```
GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EX 0 1 6 0 1.0 0.0
FR 0 1 0 0 299.792458 0.0
EN
```

**Expected parsed fields:**
- IFRQ: 0 (linear stepping)
- NFRQ: 1 (single frequency)
- FMHZ: 299.792458 MHz (= c/1m means that λ = 1 m, so half-wave dipole at 0.5 m)
- DELFRQ: 0.0

**Pass criterion:** Frequency list contains exactly one entry: 299.792458 MHz.

---

### V-PARSE-008 — FR Card, Linear Frequency Sweep

**Input:**
```
FR 0 5 0 0 100.0 50.0
```

(Partial deck shown — geometry and other cards omitted for brevity)

**Expected frequency list:**
- 100.0 MHz
- 150.0 MHz
- 200.0 MHz
- 250.0 MHz
- 300.0 MHz

**Pass criterion:** Exactly 5 frequencies, linearly spaced, starting at 100.0 MHz with 50.0 MHz step.

---

### V-PARSE-009 — FR Card, Multiplicative Frequency Sweep

**Input:**
```
FR 1 4 0 0 100.0 2.0
```

**Expected frequency list:**
- 100.0 MHz
- 200.0 MHz
- 400.0 MHz
- 800.0 MHz

**Pass criterion:** Exactly 4 frequencies, each multiplied by 2.0.

---

### V-PARSE-010 — CM/CE Comment Cards

**Input:**
```
CM Arcanum test deck
CM Half-wave dipole at 300 MHz
CE
GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EN
```

**Pass criterion:** Comment lines are silently discarded. Resulting mesh is identical to the same deck without CM/CE lines. No warnings generated.

---

## 4. Format Handling Cases (V-FMT)

### V-FMT-001 — Free-Field Format (Space Delimited)

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EN
```

Standard whitespace-delimited format. Pass criterion: fields parsed correctly.

---

### V-FMT-002 — Column-Based Format

**Input (columns strictly observed):**
```
GW     1     4     0.0     0.0    -0.25     0.0     0.0     0.25     0.001
GE 0
EN
```

Fields padded to column positions. Pass criterion: fields parsed identically to V-FMT-001. The parser must not require or be confused by column alignment.

---

### V-FMT-003 — Mixed Tabs and Spaces

**Input:**
```
GW	1	4	0.0	0.0	-0.25	0.0	0.0	0.25	0.001
GE 0
EN
```

Tab-delimited. Pass criterion: fields parsed identically to V-FMT-001.

---

### V-FMT-004 — Scientific Notation in Float Fields

**Input:**
```
GW 1 4 0.0 0.0 -2.5E-1 0.0 0.0 2.5E-1 1.0E-3
GE 0
EN
```

Pass criterion: ZW1 = -0.25, ZW2 = 0.25, RAD = 0.001. Identical result to V-FMT-001.

---

### V-FMT-005 — Windows Line Endings (CRLF)

Input deck with `\r\n` line endings rather than `\n`.

Pass criterion: Parser handles CRLF without error or spurious field misreads. A common real-world issue with decks created on Windows.

---

## 5. Card Routing Cases (V-ROUTE)

### V-ROUTE-001 — Complete Deck, All Phases Receive Correct Data

**Input:**
```
CM Test routing deck
CE
GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
FR 0 1 0 0 299.792458 0.0
EX 0 1 6 0 1.0 0.0
LD 5 1 1 11 3.72E7 0.0 0.0
RP 0 19 37 1000 0.0 0.0 10.0 10.0 0.0
EN
```

**Expected routing:**

| Card | Destination | Key Data |
|---|---|---|
| GW | Phase 1 — Mesh | 11-segment dipole, tag 1 |
| FR | Phase 2 + Phase 3 | 299.792458 MHz, single frequency |
| EX | Phase 3 | Source on tag 1, segment 6, 1V |
| LD | Phase 3 | Conductivity load on all segments of tag 1 |
| RP | Phase 4 | Far-field pattern, 19θ × 37φ, 10° spacing |

**Pass criterion:** Each phase's input structure contains exactly the data from its card(s) and nothing from other cards. Phase 1 has no knowledge of FR, EX, LD, or RP. Phase 4 has no knowledge of wire geometry directly.

---

### V-ROUTE-002 — GN Card Split

**Input:**
```
GW 1 4 0.0 0.0 0.05 0.0 0.0 0.5 0.001
GN 2 0 0 0 13.0 0.005
GE 1
EN
```

**Expected routing:**

| Data | Destination |
|---|---|
| IPERF = 2 (lossy, no images) | Phase 1: GroundType::Lossy, images_generated = false |
| EPSE = 13.0, SIG = 0.005 | Phase 2: ground electrical parameters |

**Pass criterion:** Phase 1 does not use EPSE or SIG for any computation. Phase 2 receives EPSE and SIG. The GN card is parsed once and its data is split — neither phase re-parses the raw card.

---

## 6. Hard Error Cases (V-ERR)

### V-ERR-001 — Missing GE Card

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
EN
```

**Expected:** Hard error. Parser aborts. Error message identifies missing GE terminator.

---

### V-ERR-002 — NS = 0 on GW Card

**Input:**
```
GW 1 0 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EN
```

**Expected:** Hard error. NS = 0 is invalid. Error identifies the offending card and field.

---

### V-ERR-003 — Zero-Length Wire

**Input:**
```
GW 1 4 0.5 0.5 0.5 0.5 0.5 0.5 0.001
GE 0
EN
```

**Expected:** Hard error. End 1 and end 2 are identical. Error identifies the card.

---

### V-ERR-004 — Duplicate ITAG

**Input:**
```
GW 1 4 0.0 0.0 -0.5 0.0 0.0 0.0 0.001
GW 1 4 0.0 0.0  0.0 0.0 0.0 0.5 0.001
GE 0
EN
```

**Expected:** Hard error. Tag 1 appears on two GW cards. Error identifies both cards and the duplicate tag.

---

### V-ERR-005 — EX References Unknown Tag

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EX 0 99 2 0 1.0 0.0
EN
```

Tag 99 does not exist in the mesh.

**Expected:** Hard error. Error identifies the EX card and the unknown tag number.

---

### V-ERR-006 — EX Segment Number Out of Range

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EX 0 1 9 0 1.0 0.0
EN
```

Tag 1 has 4 segments. ISEG = 9 is out of range.

**Expected:** Hard error. Error identifies the EX card, the tag, and the invalid segment number.

---

### V-ERR-007 — Non-Numeric Field

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 OOPS 0.001
GE 0
EN
```

**Expected:** Hard error. Field 8 (ZW2) cannot be parsed as a float. Error identifies the card, field position, and the offending token.

---

### V-ERR-008 — Multiple GN Cards

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 1
GN 2 0 0 0 13.0 0.005
GE 1
EN
```

**Expected:** Hard error. Only one GN card is permitted per deck.

---

### V-ERR-009 — Geometry Card After GE

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
GW 2 4 1.0 0.0 -0.25 1.0 0.0 0.25 0.001
EN
```

GW card appears after GE.

**Expected:** Hard error. Geometry cards must precede GE.

---

## 7. Warning Cases (V-WARN)

### V-WARN-001 — Unknown Card Type

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
XX 0 0 0
EN
```

**Expected:** Mesh produced from GW card. ParseWarnings contains one entry: unknown card `XX` at line 3. Parse does not abort.

---

### V-WARN-002 — Unsupported EX Type (Plane Wave)

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
EX 1 0 0 0 0.0 90.0
EN
```

EXTYPE = 1 (incident plane wave) is not supported in initial implementation.

**Expected:** Warning emitted identifying the unsupported EXTYPE. EX card skipped. No source is added to Phase 3 input. Mesh is still produced.

---

### V-WARN-003 — NRADL > 0 in GN Card

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 1 32
GE 1
EN
```

NRADL = 32 (radial wire ground screen) is not supported in initial implementation.

**Expected:** Warning emitted. NRADL value stored but not acted upon. Ground plane treated as PEC without radial screen.

---

### V-WARN-004 — Missing EN Card

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
FR 0 1 0 0 150.0 0.0
```

No EN card. Common in real-world decks.

**Expected:** Warning emitted noting absent EN card. Mesh and simulation parameters are still produced. Parse does not abort.

---

### V-WARN-005 — Unsupported Card (TL, NT, etc.)

**Input:**
```
GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
TL 1 6 2 6 50.0 0.0 0.0 0.0 0.0 0.0
EN
```

TL (transmission line) is not supported in initial implementation.

**Expected:** Warning emitted identifying TL card as unsupported. Card skipped. Mesh produced without transmission line.

---

## 8. Real-World Reference Decks (V-REAL)

These cases use complete, realistic `.nec` decks representative of common antenna models. They test the parser against real-world input rather than constructed minimal cases.

### V-REAL-001 — Classic Half-Wave Dipole

A standard center-fed half-wave dipole at 300 MHz, 11 segments, source at center.

**Purpose:** The simplest complete simulation deck. Every parser implementation should handle this correctly. If V-REAL-001 fails, nothing else is worth testing.

**Pass criterion:** Mesh contains 11 segments, 1 junction at center, source on segment 6, frequency 300 MHz. All fields parsed correctly with no errors or warnings.

The full deck is committed as `docs/nec-import/reference-decks/half-wave-dipole.nec`.

---

### V-REAL-002 — 3-Element Yagi-Uda

A driven element, reflector, and one director. Multiple wires, multiple tags, single source.

**Purpose:** Tests multi-wire parsing, tag uniqueness, and source referencing across a more complex geometry.

**Pass criterion:** Mesh contains 3 wires, correct segment counts per wire, source on driven element only, all tags unique and correctly mapped.

The full deck is committed as `docs/nec-import/reference-decks/yagi-3el.nec`.

---

### V-REAL-003 — Axial-Mode Helix Over Ground Plane

A 5-turn axial-mode helix above a PEC ground plane, with ground screen omitted (NRADL = 0).

**Purpose:** Tests GH + GN interaction. The primary ORI use case. Exercises the full geometry path — helix discretization, PEC ground, image generation — through the parser.

**Pass criterion:** GH fields correctly parsed and mapped to parametric helix parameters; GN IPERF = 1 triggers image generation in Phase 1; no errors; NRADL = 0 so no ground screen warning.

The full deck is committed as `docs/nec-import/reference-decks/helix-over-ground.nec`.

---

## 9. Reference Decks

Reference deck files are committed to `docs/nec-import/reference-decks/`. Each file is a valid `.nec` deck whose expected parse output is documented in this file. Reference decks serve two purposes:

1. **Regression testing** — the parser must produce the same output on every run
2. **Community validation** — real-world decks from the NEC/EZNEC community, verifiable against published results

Reference deck files must include a CM comment header identifying the source, the antenna type, and the expected key results.

---

## 10. Validation Procedure

Each case must be implemented as a Rust unit test. Tests must:

1. Construct the input as a string literal or load from a reference deck file
2. Call the parser entry point
3. Assert on the specific fields or structures specified in the expected output
4. For hard error cases, assert the function returns `Err(...)` with an error type matching the expected error category
5. For warning cases, assert `ParseWarnings` is non-empty and contains the expected warning type
6. For routing cases, assert that each phase's input structure contains exactly the expected data

---

## 11. References

- `docs/nec-import/card-reference.md` — field definitions used in all cases above
- `docs/nec-import/design.md` — parser architecture
- `docs/phase1-geometry/validation.md` — downstream geometry validation; distinct scope from this document
- Burke & Poggio, *NEC-2 Method of Moments Code* (1981) — reference for card format and field semantics

---

*Arcanum — Open Research Institute*
