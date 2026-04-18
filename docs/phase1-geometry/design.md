# Phase 1 — Geometry Design

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/design.md`  
**Status:** DRAFT  
**Revision:** 0.2

---

## 1. Purpose

Phase 1 accepts a `.nec` input deck and produces a complete, validated segment mesh. This is the internal geometric representation that all subsequent phases consume. Phase 1 owns everything spatial: wire geometry, segment discretization, junction connectivity, and the geometric boundary conditions imposed by the ground plane.

Phase 1 does not perform any electromagnetic computation. It knows nothing about frequency, current, or fields. Its only concern is answering the question: *what is the physical structure we are about to analyze?*

---

## 2. Inputs

A `.nec` input deck, read from file or passed as a string. Phase 1 processes the following NEC card types:

| Card | Description |
|---|---|
| `GW` | Straight wire |
| `GA` | Circular arc |
| `GH` | Helix |
| `GM` | Geometry move / rotate / scale |
| `GS` | Global geometry scale |
| `GE` | Geometry end (required terminator) |
| `GN` | Ground definition (partial — see Section 6) |

All other cards are parsed and stored for downstream phases but are not processed by Phase 1. Unknown cards produce a warning.

---

## 3. Outputs

Phase 1 produces a `Mesh` struct that is the sole input to Phase 2. The `Mesh` contains:

- **Segment list** — ordered list of `Segment` structs (see Section 4)
- **Junction map** — connectivity graph of which segments share endpoints (see Section 5)
- **Ground descriptor** — geometric ground boundary condition (see Section 6)
- **Tag map** — mapping from NEC wire tag numbers to segment index ranges, preserved for Phase 3 source and load assignment

The `Mesh` is immutable after Phase 1 completes. No downstream phase modifies it.

---

## 4. Segment Representation

### 4.1 The Segment Struct

Every wire in the input deck is discretized into one or more `Segment`s. A `Segment` is a curved cylindrical tube described parametrically. The parametric representation is defined in `math.md`. This document describes the fields only.

```
Segment {
    // Geometry
    curve_type: CurveType,       // Linear, Arc, or Helix
    params: CurveParams,         // Parametric description (see math.md)
    radius: f64,                 // Wire radius (meters)
    
    // Material
    material: Material,          // PEC (default), or conductivity + permeability
    
    // Bookkeeping
    tag: u32,                    // NEC wire tag number (from source card)
    segment_index: usize,        // Global index in the mesh segment list
    wire_index: usize,           // Which wire this segment belongs to
}
```

### 4.2 Curve Types

Three curve types are supported in this phase, corresponding to the three geometry cards that describe curved or straight wire. They are:

**Linear** — straight wire segment, from `GW`. A degenerate case of the general parametric representation. Included because `GW` is the most common card and must be handled correctly. See `math.md` for the parametric form.

**Arc** — circular arc segment, from `GA`. Defined by radius, start angle, and end angle in a plane. The full arc is discretized into N segments, each of which is a short arc. See `math.md`.

**Helix** — helical segment, from `GH`. Defined by pitch, radius, and number of turns. Discretized into N segments, each a short helical arc. This is the curve type where CMoM most visibly outperforms NEC-2 and is high priority for ORI use cases. See `math.md`.

### 4.3 Discretization

Each wire card specifies the number of segments N explicitly. Phase 1 does not choose N. It respects what the `.nec` file specifies. Guidelines on choosing N relative to wavelength are a "you problem". Phase 1 only enforces that N ≥ 1.

One exception: if N is specified as 0 in the input deck, Phase 1 must return a parse error, not silently produce an empty wire. Because we're not cruel. Negative numbers of wires produces an Easter Egg.

---

## 5. Junction Handling

### 5.1 What a Junction Is

A junction is a point in space where two or more wire endpoints meet. Junctions are how antenna structures are connected. Such as a T-match, a driven element fed at the center, a Yagi boom with elements attached. NEC files do not declare junctions explicitly. They are inferred by finding wire endpoints that are geometrically coincident within a tolerance.

### 5.2 What a Junction Is Not

Not every pair of coincident endpoints qualifies as a junction. Within a single wire card (`GW`, `GA`, or `GH`), the end of segment k and the start of segment k+1 are always connected — that is the definition of discretization. These **intra-wire adjacent boundaries** are implicit in the segment ordering and are never recorded as explicit junctions in the junction map.

Only two cases produce a junction record:

1. **Cross-wire connection** — an endpoint of one wire card is geometrically coincident with an endpoint of a different wire card.
2. **Self-loop closure** — the last segment end of a wire is coincident with its own first segment start (a closed loop antenna such as `GA` with 360°).

This distinction matters for valence counting: the midpoint of a 2-segment dipole (`GW 1 2 ...`) has no junction record even though two segment endpoints meet there. It is addressable by an `EX` card via `(tag, segment)` directly.

### 5.3 Junction Detection

Two endpoints are considered coincident if their distance is less than a tolerance `ε`. The default tolerance is:

```
ε = min(radius_a, radius_b) × 0.01
```

This is intentionally conservative. Phase 1 should warn when endpoints are close but not coincident (distance between `ε` and `10ε`), as this often indicates a modeling error in the input deck. Because we are helpful.

### 5.4 The Junction Map

The junction map records, for each junction point, the list of segment endpoints that meet there. This is the connectivity graph Phase 2 uses to enforce current continuity at junctions (Kirchhoff's current law in the MoM formulation).

Each junction is assigned a unique index. The map is bidirectional. Given a segment endpoint, you can look up which junction it belongs to. Given a junction, you can enumerate all connected segment endpoints.

### 5.5 Degenerate Cases

- **Isolated wire endpoint** — an endpoint that belongs to no junction. Valid — monopoles and open-ended dipoles have free endpoints.
- **Two-wire junction** — the normal case for connected antennas.
- **Three-or-more-wire junction** — valid, occurs in log-periodic arrays, feed networks, and complex structures. Phase 1 must handle arbitrary valence. Valence is counted in *wires*, not segment endpoints: a wire that passes through a junction (its midpoint is at the junction point) contributes 1 to valence even though it places 2 segment endpoints into the junction record.
- **Self-loop** — a wire whose start and end endpoints are coincident. Valid for loop antennas. Phase 1 should detect and flag these explicitly rather than treating them as a two-wire junction.

Any other problem cases that may come up in development that we missed would go in the above list. 

---

## 6. Ground Plane

### 6.1 The GN Card Split

The `GN` card carries two conceptually distinct pieces of information that are consumed by different phases.

**Geometric ground (Phase 1 concern):** Whether an infinite ground plane exists at z = 0, and whether it is treated as a perfect electrical conductor (PEC) for the purpose of image theory. If a PEC ground is specified, Phase 1 generates mirror images of all wire segments reflected through the z = 0 plane. These image segments are added to the mesh as a geometric fact. Phase 2 receives a mesh that already includes them and does not need to know they are images. Isnt't that cool?

**Lossy ground parameters (Phase 2 concern):** The conductivity (σ) and relative permittivity (εᵣ) of the ground, which modify the Green's function via the Sommerfeld integral formulation. Phase 1 parses and stores these values in the `GroundDescriptor` struct and passes them downstream, but does not use them for itself.

This split is intentional and must be maintained. Phase 1 must not reach into electromagnetic computation. That's not what it is about. Phase 2 must not re-parse the `GN` card. That is not what it is about. 

### 6.2 Ground Descriptor Struct

```
GroundDescriptor {
    ground_type: GroundType,    // None, PEC (perfect), or Lossy
    conductivity: Option<f64>,  // σ (S/m) — None if PEC or no ground
    permittivity: Option<f64>,  // εᵣ (relative) — None if PEC or no ground
    images_generated: bool,     // True if Phase 1 added image segments to mesh
}
```

### 6.3 Image Generation

When `ground_type` is `PEC`, Phase 1 reflects every wire segment through z = 0. The image segment has:

- Reflected geometry (z is -z for all points)
- Same radius and material as the original
- A tag that marks it as an image (not addressable by NEC source or load cards)
- An entry in the junction map that correctly connects the image to the original at the ground plane

Wires that lie exactly in the z = 0 plane are not reflected. They are actually their own images. Phase 1 must detect and handle this case without producing duplicate segments. Otherwise you get a sci-fi movie plot. 

### 6.4 No Ground

If no `GN` card is present, the default is free space. There is no ground plane, no images. The `GroundDescriptor` is set to `GroundType::None`. This is a useful case for analysis and visualization.

---

## 7. Geometry Transformations

### 7.1 GM — Move / Rotate / Scale

The `GM` card applies a translation, rotation, and/or scaling to a subset of wire tags, optionally with repetition (generating multiple translated/rotated copies). This is commonly used to build arrays, helical antennas with multiple turns defined from a single turn, and rotational structures.

Phase 1 applies `GM` transformations. The resulting geometry is resolved into concrete segment positions before the mesh is finalized. The mesh contains no transformation records, only the final geometry. The transformations are consumed along the way. 

### 7.2 GS — Global Scale

The `GS` card applies a uniform scale factor to all wire coordinates. Applied before any `GM` transformations. Used to convert between unit systems (e.g. a model built in wavelengths scaled to meters at a given frequency).

**Important:** `GS` scales wire coordinates but not wire radii. This is consistent with NEC-2 behavior and must be explicitly documented in the user-facing card reference.

---

## 8. Error Handling

Phase 1 must distinguish between errors that should abort parsing and warnings that should be recorded but allow the mesh to be produced. Basic error level stuff here.

**Hard errors (abort):**
- Missing `GE` terminator
- Segment count N = 0 on any wire card
- Wire with zero length
- Malformed card (wrong number of fields, non-numeric where numeric expected)
- Coordinate system violation (wire endpoint with NaN or infinite coordinate)
- Anything else development reveals

**Warnings (record and continue):**
- Unknown card type
- Near-coincident endpoints that do not meet junction tolerance
- Wire radius of zero (degenerate but technically parseable)
- `GM` referencing a tag that does not exist
- Anything else development reveals

All warnings are collected into a `ParseWarnings` list returned alongside the `Mesh`. Callers can inspect or ignore warnings. Phase 1 never silently discards information.

---

## 9. Interface to Downstream Phases

The `Mesh` struct is the sole output of Phase 1 and the sole geometric input to all subsequent phases. The interface contract is:

- **Phase 2** consumes the full segment list and ground descriptor to compute the impedance matrix
- **Phase 3** consumes the tag map to locate segments referenced by `EX` (source) and `LD` (load) cards
- **Phase 4** consumes the segment list for field integration and the ground descriptor for image contributions to far-field patterns

No downstream phase calls back into Phase 1. No downstream phase re-parses the `.nec` file. 

---

## 10. Relationship to NEC Import

Phase 1 implements the geometry portion of the NEC import pipeline described in `docs/nec-import/design.md`. The full card routing table and parser architecture are specified there. This document specifies only what Phase 1 does with the geometry cards it receives. This document does not cover how the parser works.

---

## 11. Open Questions

The following questions are unresolved and should be discussed before `design.md` gets to be final.

1. **Spline wires** — NEC has no spline card. Should Phase 1 include a programmatic API for spline-defined wires for use from the Python layer, even though there is no `.nec` equivalent? Decision deferred pending native format discussion. Really don't want to have to have a native format because translations are full of error potential.

2. **Segmentation guidelines** — Should Phase 1 emit a warning when a segment length exceeds λ/10 at the highest frequency in the `FR` card? This requires Phase 1 to be aware of the frequency, which it currently is not. Alternative: make this a Python-layer check? Kind of ugly though. 

3. **Non-planar arcs** — `GA` in NEC defines arcs in a specific plane. Should we just rotate the plane to get other arcs? Is it that easy? Why isn't this in the card deck already?

---

## 12. References

- `docs/phase1-geometry/math.md` — parametric curve representations for Linear, Arc, and Helix segment types
- `docs/phase1-geometry/validation.md` — geometric test cases and continuity checks
- `docs/nec-import/design.md` — full NEC parser architecture and card routing
- `docs/nec-import/card-reference.md` — field-by-field reference for all supported cards
- Burke & Poggio, *NEC-2 Method of Moments Code* (1981) — authoritative NEC card definitions! This is what we are basing off of.

---

*Arcanum — Open Research Institute*
