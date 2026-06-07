# Phase 1 — Geometry Implementation Plan

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/plan.md`  
**Status:** APPROVED — ready for implementation  
**Revision:** 0.1

---

## Overview

Phase 1 consumes `MeshInput` from `arcanum-nec-import` and produces a `Mesh`
— the complete, validated, discretized segment representation consumed by all
subsequent phases. It owns everything spatial: wire discretization, geometry
transformations, junction detection, PEC image generation, and the tag map.

**Crate:** `crates/arcanum-geometry`  
**Input:** `arcanum_nec_import::MeshInput`  
**Output:** `Mesh`

---

## Step 1 — Data structures (`src/lib.rs` + `src/mesh.rs`)

Define the public output types:

- `Segment` — curve type, parametric params, radius, material, tag, indices
- `CurveType` / `CurveParams` — enum covering `Linear`, `Arc`, `Helix`
- `Junction` — index, list of connected segment endpoints, self-loop flag
- `Mesh` — segment list, junction map, ground descriptor, tag map
- `GroundDescriptor` — ground type, conductivity, permittivity, images_generated flag
- `TagMap` — tag → segment index range

---

## Step 2 — Linear discretization (`src/discretize.rs`)

Implement `discretize_straight(wire: &StraightWire) -> Vec<Segment>`.

Each of the N segments gets endpoints computed from the closed-form parametric
expression `r(t) = r_a + t(r_b - r_a)` at `t = k/N` and `t = (k+1)/N`.

Tests: V-LIN-001 through V-LIN-004.

---

## Step 3 — Arc discretization (`src/discretize.rs`)

Implement `discretize_arc(wire: &ArcWire) -> Vec<Segment>`.

Angles converted from degrees to radians. Each segment k spans `θ₁ₖ` to
`θ₂ₖ` computed from the closed-form expression — no incremental accumulation.

Tests: V-ARC-001 (XZ plane, 4 segments), V-ARC-002 (full circle closure),
V-ARC-003 (near-coincident endpoint warning).

---

## Step 4 — Helix discretization (`src/discretize.rs`)

Implement `discretize_helix(wire: &HelixWire) -> Vec<Segment>`.

Both uniform and tapered cases from `math.md` Sections 5.2–5.4. Endpoints
evaluated from closed-form at `t = k/N` — never accumulated.

Tests: V-HEL-001 (single turn), V-HEL-002 (5-turn continuity, gap < 1×10⁻¹²
m), V-HEL-003 (image generation).

---

## Step 5 — GS and GM transformations (`src/transforms.rs`)

Consume `MeshInput.transforms` after discretization:

1. Apply `gs_scale` to all segment endpoint coordinates (not wire radii)
2. Apply each `GmOperation` in order:
   - `n_copies == 0`: rotate then translate existing segments in place
   - `n_copies > 0`: keep originals, generate N transformed copies with
     incremented tags, append to segment list and tag map

Rotation order: ROX → ROY → ROZ (NEC2 convention).

Tests: V-TRF-001 (GS scale, radius unchanged), V-TRF-002 (GM translation).

---

## Step 6 — Junction detection (`src/junctions.rs`)

For every pair of segment endpoints, compute distance. Tolerance:
`ε = min(radius_a, radius_b) × 0.01`.

- Distance < ε → same junction (merge)
- Distance between ε and 10ε → warning (near-coincident, no junction created)
- Full-circle arc (ANG2 − ANG1 = 360°) → self-loop junction flagged explicitly

Build bidirectional map: segment endpoint → junction index, junction index →
list of endpoints.

Tests: V-LIN-003 (two-wire junction), V-LIN-004 (T-junction, valence 3),
V-ARC-002 (self-loop), V-ARC-003 (near-coincident warning).

---

## Step 7 — PEC image generation (`src/images.rs`)

When `ground_type == PEC`:

- For each segment not lying in z = 0: create image segment with z-coordinates
  negated
- Wires entirely in z = 0 plane: no image generated, warning emitted
- Image segments appended after all real segments; flagged in tag map as
  non-addressable
- Junction map updated to connect image endpoints to originals at z = 0

Tests: V-GND-001 (no ground), V-GND-002 (PEC, 4+4 segments), V-GND-003
(lossy params stored, no images), V-GND-004 (wire in ground plane).

---

## Step 8 — Tag map (`src/tagmap.rs`)

Build `TagMap` mapping each tag to its contiguous range of segment indices.
Updated as GM copy segments are appended. Must be consistent with what the
nec-import tag registry registered for EX/LD validation.

Test: V-TAG-001.

---

## Step 9 — Public API (`src/lib.rs`)

```rust
pub fn build_mesh(
    input: MeshInput,
    ground_electrical: Option<GroundElectrical>,
) -> Result<(Mesh, GeometryWarnings), GeometryError>
```

Single entry point. Calls Steps 2–8 in order. Returns the `Mesh` and any
accumulated warnings. `GeometryError` mirrors the style of `ParseError`.

`ground_electrical` carries the lossy ground parameters from the GN card
(conductivity and permittivity). These live in `SimulationInput`, not in
`MeshInput`, so they are passed separately. Pass `None` for PEC or free-space
models.

`GeometryWarnings` is distinct from nec-import's `ParseWarnings`; it carries
Phase 1-specific warning types (`NearCoincidentEndpoints`, `WireInGroundPlane`).

---

## Step 10 — Tests (`src/tests/`)

One test file per validation category:

| File | Cases |
|---|---|
| `tests/linear.rs` | V-LIN-001 through V-LIN-004 |
| `tests/arc.rs` | V-ARC-001 through V-ARC-003 |
| `tests/helix.rs` | V-HEL-001 through V-HEL-003 |
| `tests/transforms.rs` | V-TRF-001 through V-TRF-002 |
| `tests/ground.rs` | V-GND-001 through V-GND-004 |
| `tests/tagmap.rs` | V-TAG-001 |
| `tests/warnings.rs` | V-WARN-001 through V-WARN-002 |

---

## Implementation order

```
Step 1  (data structures)
  → Step 2  (linear)        + V-LIN tests
  → Step 3  (arc)           + V-ARC tests
  → Step 4  (helix)         + V-HEL tests
  → Step 5  (transforms)    + V-TRF tests
  → Step 6  (junctions)     + junction tests
  → Step 7  (images)        + V-GND tests
  → Step 8  (tag map)       + V-TAG test
  → Step 9  (public API)
  → Step 10 (warning tests)
```

---

*Arcanum — Open Research Institute*
