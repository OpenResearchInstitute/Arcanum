# Phase 1 — Implementation Changes and Open Items

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/phase1-changes.md`  
**Status:** CLOSED — 2026-04-17  
**Revision:** 0.2

---

## Purpose

This document records deviations from the original Phase 1 specification that
occurred during implementation, and the documentation changes made before
Phase 1 was closed.

---

## 1. API Change — `build_mesh` Signature

### What the plan specified

`plan.md` Step 9 specified:

```rust
pub fn build_mesh(input: MeshInput) -> Result<(Mesh, ParseWarnings), GeometryError>
```

### What was actually implemented

```rust
pub fn build_mesh(
    input: MeshInput,
    ground_electrical: Option<GroundElectrical>,
) -> Result<(Mesh, GeometryWarnings), GeometryError>
```

### Why it changed

**Extra parameter — `ground_electrical: Option<GroundElectrical>`**

The lossy ground electrical parameters (conductivity σ, permittivity εᵣ) live
in `SimulationInput.ground_electrical` in the nec-import layer, not in
`MeshInput`. When the NEC GN card is parsed, the geometric part (PEC vs. lossy
vs. free space) goes into `MeshInput.ground`, and the electrical parameters go
into a separate `GroundElectrical` struct in `SimulationInput`.

Phase 1 needs the electrical parameters to populate the `GroundDescriptor` it
returns — but `MeshInput` does not carry them. Adding `ground_electrical` as a
separate `Option<GroundElectrical>` parameter was the clean fix: the caller
passes `None` for free-space and PEC cases, and `Some(ge)` when the GN card
specified lossy parameters.

**`ParseWarnings` → `GeometryWarnings`**

The original plan reused the nec-import `ParseWarnings` name, but Phase 1 has
its own warning types (`NearCoincidentEndpoints`, `WireInGroundPlane`) that do
not exist in the nec-import layer. Using a distinct `GeometryWarnings` type
keeps the phases independent and avoids importing nec-import error types into
the geometry crate's public API.

**Closed** — `plan.md` Step 9 corrected 2026-04-17.

---

## 2. Junction Semantics — Behaviour Not in Original Spec

### What was underspecified

`design.md` Section 5 defined what a junction is but did not define what it is
not. `validation.md` V-LIN-001 described a "Junction at (0, 0, 0.0)"
at the midpoint of a 2-segment wire, which contradicts V-LIN-003's claim of
"1 junction" for a two-wire model where each wire has 3 segments (which would
produce 5 junctions if intra-wire boundaries counted).

### Decision made during implementation

Intra-wire adjacent segment boundaries are **not** junctions. Only:

1. Cross-wire connections — an endpoint of one wire card coincides with an
   endpoint of a different wire card.
2. Self-loop closures — the last segment end of a wire coincides with its
   own first segment start (e.g. a 360° arc).

This is the interpretation consistent with V-LIN-003, V-LIN-004, V-ARC-002,
and V-ARC-003. V-LIN-001's junction claim was incorrect.

### Doc fixes made

- `design.md` (→ Revision 0.2): Added Section 5.2 "What a Junction Is Not";
  renumbered 5.2–5.4 to 5.3–5.5; added valence-counting clarification to 5.5;
  fixed "Becasue" typo.
- `validation.md` (→ Revision 0.3): Corrected V-LIN-001 (no junction at
  midpoint); added intra-wire note to V-LIN-003; updated V-LIN-004 to explain
  4 endpoints / 3 wires / valence 3.

**Closed** — 2026-04-17.

---

## 3. Documentation Fixes

### 3.1 Incorrect GH Card Strings in `validation.md`

**Affected cases:** V-HEL-001, V-HEL-002, V-HEL-003

The GH card strings had the wrong number of fields and an inconsistent
`total_length` value. Corrected cards:

| Case | Correct card |
|---|---|
| V-HEL-001 (1 turn, 8 segs) | `GH 1 8 0.0628 0.0628 0.05 0.05 0.001` |
| V-HEL-002 (5 turns, 40 segs) | `GH 1 40 0.0628 0.314 0.05 0.05 0.001` |
| V-HEL-003 (2 turns, 16 segs) | `GH 1 16 0.0628 0.1256 0.05 0.05 0.001` |

**Closed** — corrected 2026-04-17.

---

### 3.2 Outdated API Signature in `plan.md`

`plan.md` Step 9 updated to show the actual `build_mesh` signature with the
`ground_electrical` parameter and `GeometryWarnings` return type.

**Closed** — corrected 2026-04-17.

---

### 3.3 Convergence Plots Requirement in `validation.md` Section 10

The requirement for convergence plots (V-HEL-002 and V-ARC-001) was removed.
The closed-form parametric implementation makes geometric continuity a
mathematical certainty, not an empirical observation. See `plots.md` for the
full analysis.

**Closed** — Option 3 approved and removed from `validation.md` 2026-04-17.

---

## 4. Bug Found Post-Closing — `images.rs` `wire_index`

**Discovered:** 2026-04-17, during Python binding work  
**File:** `crates/arcanum-geometry/src/images.rs`, line 43  

### The bug

Image segments were assigned `wire_index: i` (the loop counter over real
segments) instead of `wire_index: seg.wire_index`. For a single-wire helix
with 40 segments over PEC ground, this gave image segments wire indices 0–39
instead of all 0, causing adjacent image segments to appear as cross-wire
connections and producing 40 false junctions.

### Why existing tests did not catch it

V-GND-002, V-GND-004, and V-HEL-003 did not assert on junction counts. The
false junctions were present but unobserved.

### Fix

Changed `wire_index: i` to `wire_index: seg.wire_index`. The loop variable `i`
was also removed (changed `for (i, seg) in ... .enumerate()` to
`for seg in ...`).

### Test coverage added

Junction count assertions added to three tests (`validation.md` → Revision 0.4):

| Test | Expected junctions | Reason |
|---|---|---|
| V-GND-002 | 1 | Real seg 0 Start and image seg 4 Start share z = 0 |
| V-GND-004 | 0 | No images; single wire; no cross-wire connections |
| V-HEL-003 | 1 | Real seg 0 Start and image seg 16 Start share z = 0 |

**Closed** — fixed and tested 2026-04-17.

---

## 5. Summary Table

| # | Item | Status |
|---|---|---|
| 1 | `build_mesh` signature change | **Closed** — plan.md corrected 2026-04-17 |
| 2 | Junction semantics | **Closed** — design.md and validation.md corrected 2026-04-17 |
| 3.1 | GH card strings in validation.md | **Closed** — corrected 2026-04-17 |
| 3.2 | plan.md API signature | **Closed** — corrected 2026-04-17 |
| 3.3 | Convergence plots requirement | **Closed** — Option 3, removed from validation.md 2026-04-17 |
| 4 | `images.rs` `wire_index` bug | **Closed** — fixed and tested 2026-04-17 |

---

*Arcanum — Open Research Institute*
