# Phase 1 — Geometry Validation

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/validation.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the validation cases for Phase 1 geometry discretization. Each case specifies:

- The input (a `.nec` card sequence or programmatic description)
- The expected output (concrete values the `Mesh` struct must produce)
- The pass/fail criterion

These cases are the ground truth against which the Phase 1 implementation is tested. `math.md` provides the derivations that justify the expected output values. This document specifies only what the values must be.

All expected values in this document are exact or carry an explicit numerical tolerance. Ambiguous expectations are a defect in this document, not an acceptable implementation state. If you see something, say something! Quality happens when community standards are enforced.

---

## 2. Coordinate System and Units

All coordinates are in meters. The coordinate system is right-handed Cartesian with z pointing up. The ground plane, when present, is at z = 0. These conventions must hold for all test cases. This is The Way.

---

## 3. Linear Segment Cases (GW)

### V-LIN-001 — Single Straight Wire, Two Segments

**Input:**
```
GW 1 2 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
```

A center-fed half-wave dipole along the z-axis, 0.5 m long, 1 mm radius, 2 segments.

**Expected Mesh:**
- Segment count: 2
- Segment 0: start (0, 0, -0.25), end (0, 0, 0.0), length 0.25 m
- Segment 1: start (0, 0, 0.0), end (0, 0, 0.25), length 0.25 m
- Both segments: radius 0.001 m, material PEC, tag 1
- Junction at (0, 0, 0.0) connecting segment 0 end to segment 1 start
- Two free endpoints: (0, 0, -0.25) and (0, 0, 0.25)
- Ground descriptor: GroundType::None

**Tolerance:** Endpoint coordinates exact to double precision. Length exact to double precision.

---

### V-LIN-002 — Single Straight Wire, One Segment

**Input:**
```
GW 1 1 0.0 0.0 0.0 1.0 0.0 0.0 0.005
GE 0
```

A single segment along the x-axis, 1 m long, 5 mm radius.

**Expected Mesh:**
- Segment count: 1
- Segment 0: start (0, 0, 0), end (1, 0, 0), length 1.0 m
- No junctions (single segment, two free endpoints)
- Tag map: tag 1 → segment index [0]

**Tolerance:** Coordinates exact to double precision.

---

### V-LIN-003 — Two Wires Joined at a Junction

**Input:**
```
GW 1 3 0.0 0.0 0.0 1.0 0.0 0.0 0.001
GW 2 3 1.0 0.0 0.0 2.0 0.0 0.0 0.001
GE 0
```

Two collinear wires sharing an endpoint at (1, 0, 0).

**Expected Mesh:**
- Segment count: 6
- Junction at (1.0, 0.0, 0.0) connecting segment 2 (end) to segment 3 (start)
- Junction map: 1 junction, valence 2
- Free endpoints: (0, 0, 0) and (2, 0, 0)

---

### V-LIN-004 — T-Junction, Three Wires Meeting at a Point

**Input:**
```
GW 1 2 -0.5 0.0 0.0  0.5 0.0 0.0 0.001
GW 2 2  0.0 0.0 0.0  0.0 0.5 0.0 0.001
GW 3 2  0.0 0.0 0.0  0.0 0.0 0.5 0.001
GE 0
```

Three wires meeting at the origin, one along each axis.

**Expected Mesh:**
- Segment count: 6
- Junction at (0, 0, 0) with valence 3 — wire 1 midpoint, wire 2 start, wire 3 start
- Junction map correctly identifies all three segment endpoints at origin as the same junction

**Note:** This case validates that Phase 1 correctly handles junctions of valence > 2.

---

### V-LIN-005 — Zero-Length Wire (Hard Error)

**Input:**
```
GW 1 4 0.5 0.5 0.5 0.5 0.5 0.5 0.001
GE 0
```

Start and end coordinates are identical.

**Expected result:** Hard error. Parse must abort with a descriptive error message identifying the card and the zero-length condition. No `Mesh` is returned.

---

### V-LIN-006 — Zero Segment Count (Hard Error)

**Input:**
```
GW 1 0 0.0 0.0 0.0 1.0 0.0 0.0 0.001
GE 0
```

**Expected result:** Hard error. N = 0 is invalid. Parse must abort.

---

### V-LIN-007 — Missing GE Terminator (Hard Error)

**Input:**
```
GW 1 4 0.0 0.0 -0.5 0.0 0.0 0.5 0.001
```

No `GE` card.

**Expected result:** Hard error. Parse must abort with a message identifying the missing terminator.

---

## 4. Arc Segment Cases (GA)

### V-ARC-001 — Semicircular Arc, 4 Segments

**Input:**
```
GA 1 4 0.5 0.0 180.0 0.001
GE 0
```

A semicircular arc of radius 0.5 m, from 0° to 180°, 4 segments, in the default plane. Here's where CMoM starts to show off.

**Expected Mesh:**
- Segment count: 4
- Each segment subtends 45° of arc
- Segment 0 start: (0.5, 0.0, 0.0)
- Segment 0 end: (cos(45°)×0.5, sin(45°)×0.5, 0.0) = (0.3536, 0.3536, 0.0)
- Segment 3 end: (-0.5, 0.0, 0.0)
- All segment radii: 0.001 m
- Arc length of each segment: π×0.5/4 = 0.3927 m

**Tolerance:** Endpoint coordinates to 4 decimal places (0.0001 m).

---

### V-ARC-002 — Full Circle (Loop Antenna), 8 Segments

**Input:**
```
GA 1 8 0.25 0.0 360.0 0.001
GE 0
```

A full circular loop of radius 0.25 m, 8 segments. 

**Expected Mesh:**
- Segment count: 8
- Start endpoint of segment 0 and end endpoint of segment 7 are coincident → self-loop junction
- Self-loop flag set on the junction
- Total arc length: 2π×0.25 = 1.5708 m
- Each segment arc length: 0.1963 m

**Note:** This case validates self-loop detection. The junction at the closure point must be flagged as a self-loop, not treated as a two-wire junction.

**Tolerance:** Endpoint coordinates to 4 decimal places.

---

### V-ARC-003 — Near-Coincident Endpoints Warning

**Input:**
```
GA 1 8 0.25 0.0 359.0 0.001
GE 0
```

An arc that almost closes (359° instead of 360°). The gap between start and end is very small but nonzero. Gotta catch 'em all. 

**Expected result:**
- Mesh is produced (not a hard error)
- A warning is emitted noting near-coincident endpoints at the gap
- No junction is created at the gap
- Two free endpoints remain

---

## 5. Helix Segment Cases (GH)

### V-HEL-001 — Single-Turn Helix, 8 Segments

**Input:**
```
GH 1 8 0.0628 0.05 0.05 0.001 0.001 0.0
GE 0
```

A single-turn helix: pitch 0.0628 m (≈ λ/10 at 480 MHz), radius 0.05 m, 8 segments, 1 mm wire radius. Simple but effective test.

**Expected Mesh:**
- Segment count: 8
- Segment 0 start: (0.05, 0.0, 0.0)
- After one full turn, end of segment 7: (0.05, 0.0, 0.0628) — same x,y as start, advanced by one pitch in z
- Each segment subtends 45° of rotation and 0.0628/8 = 0.00785 m of axial advance
- Helix is right-handed (positive pitch, positive z advance)

**Tolerance:** Endpoint coordinates to 4 decimal places.

---

### V-HEL-002 — Multi-Turn Helix, Endpoint Continuity

**Input:**
```
GH 1 40 0.0628 0.05 0.05 0.001 0.001 0.0
GE 0
```

A 5-turn helix (40 segments at 8 per turn), same parameters as V-HEL-001.

**Expected Mesh:**
- Segment count: 40
- End of segment N-1 exactly equals start of segment N for all N (geometric continuity)
- Total axial length: 5 × 0.0628 = 0.314 m
- Final endpoint z-coordinate: 0.314 m

**Primary validation target:** Geometric continuity. The gap between adjacent segment endpoints must be zero to double precision. Any nonzero gap indicates a discretization error in the helix parametrization.

**Tolerance:** Continuity gaps < 1×10⁻¹² m (double precision rounding only).

---

### V-HEL-003 — Helix Over Ground Plane, Image Generation

**Input:**
```
GH 1 16 0.0628 0.05 0.05 0.001 0.001 0.0
GN 1
GE 1
```

A 2-turn helix above a PEC ground plane.

**Expected Mesh:**
- Segment count: 32 (16 original + 16 image segments)
- Image segments have z-coordinates negated relative to originals
- Image segments are flagged as images in the tag map (not addressable by EX/LD cards)
- Ground descriptor: GroundType::PEC, images_generated: true

---

## 6. Geometry Transformation Cases (GM, GS)

### V-TRF-001 — GS Global Scale

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.01
GS 0 0 0.5
GE 0
```

Scale all coordinates by 0.5. Wire radius is NOT scaled. Make sure we don't do that.

**Expected Mesh:**
- Segment endpoints scaled by 0.5: z range now -0.125 to 0.125
- Wire radius unchanged: 0.01 m
- Total wire length: 0.25 m (was 0.5 m)

**Note:** This case explicitly validates the GS radius behavior documented in `design.md` Section 7.2.

---

### V-TRF-002 — GM Translation

**Input:**
```
GW 1 2 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GM 0 0 0.0 0.0 0.0 1.0 0.0 0.0
GE 0
```

Translate all wires by (1.0, 0.0, 0.0). Ohana means no one is left behind.

**Expected Mesh:**
- Segment endpoints shifted: x-coordinates all +1.0
- Segment 0 start: (1.0, 0.0, -0.25)
- Segment 1 end: (1.0, 0.0, 0.25)

---

## 7. Ground Plane Cases (GN)

### V-GND-001 — No Ground Card

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GE 0
```

No `GN` card present. Oops.

**Expected:** GroundDescriptor with GroundType::None. No images generated.

---

### V-GND-002 — PEC Ground, Image Generation

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 1
GE 1
```

A vertical wire above a PEC ground plane (z = 0 to z = 0.5).

**Expected:**
- Original 4 segments: z from 0.0 to 0.5
- Image 4 segments: z from 0.0 to -0.5 (reflected)
- Total segment count: 8
- Ground descriptor: GroundType::PEC, images_generated: true
- Image segment endpoints are exact reflections: if original has endpoint (x, y, z), image has (x, y, -z)

---

### V-GND-003 — Lossy Ground Parameters Stored, Not Used

**Input:**
```
GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001
GN 2 0 0 0 13.0 0.005
GE 1
```

Lossy ground: εᵣ = 13.0, σ = 0.005 S/m.

**Expected:**
- Ground descriptor: GroundType::Lossy
- conductivity: 0.005 S/m
- permittivity: 13.0
- images_generated: false (lossy ground does not use image theory)
- No image segments added to mesh
- Phase 1 does not compute anything with these values. They are stored for Phase 2

---

### V-GND-004 — Wire in Ground Plane (No Self-Image)

**Input:**
```
GW 1 4 -0.25 0.0 0.0 0.25 0.0 0.0 0.001
GN 1
GE 1
```

A horizontal wire lying exactly in the z = 0 ground plane. Trick!

**Expected:**
- No image segments generated (wire is its own image)
- Warning emitted noting wire lies in ground plane
- Segment count: 4 (not 8)

---

## 8. Tag Map Cases

### V-TAG-001 — Multiple Wires, Tag Map Correctness

**Input:**
```
GW 1 3 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
GW 2 5 0.0 0.0  0.5  1.0 0.0 0.5  0.001
GE 0
```

**Expected Tag Map:**
- Tag 1 indicates segment indices [0, 1, 2]
- Tag 2 indicates segment indices [3, 4, 5, 6, 7]
- Total segments: 8

---

## 9. ParseWarnings Cases

### V-WARN-001 — Unknown Card Type

**Input:**
```
GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
XX 0 0
GE 0
```

`XX` is not a valid NEC card.

**Expected:**
- Mesh is produced from the `GW` card
- `ParseWarnings` contains one entry identifying card `XX` as unknown
- Parse does not abort

---

### V-WARN-002 — Near-Coincident Endpoints

Two wire endpoints separated by a distance between ε and 10ε (see `design.md` Section 5.2).

**Expected:**
- Mesh is produced
- Warning emitted with coordinates of both near-coincident endpoints and their separation distance
- No junction created at the near-coincident points

---

## 10. Validation Procedure

Each case must be implemented as a Rust unit test in the Phase 1 test module. Tests must:

1. Construct the input as a string literal or programmatic struct
2. Call the Phase 1 parse/discretize entry point
3. Assert the expected output values with the specified tolerances using `assert!` or `approx_eq!`
4. For hard error cases, assert that the function returns `Err(...)` not `Ok(...)`
5. For warning cases, assert that `ParseWarnings` is non-empty and contains the expected warning type

Convergence plots, demonstrating that geometric continuity and endpoint accuracy hold as N increases, are required for V-HEL-002 and V-ARC-001 before Phase 1 implementation is marked complete.

---

## 11. References

- `docs/phase1-geometry/design.md` — struct definitions, junction detection algorithm, GN split
- `docs/phase1-geometry/math.md` — parametric equations used to derive expected endpoint coordinates
- `docs/nec-import/card-reference.md` — field definitions for GW, GA, GH, GM, GS, GN, GE cards

---

*Arcanum — Open Research Institute*
