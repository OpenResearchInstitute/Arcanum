# Phase 1 — Geometry Mathematics

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/math.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the parametric mathematical representations for the three curve types used in Arcanum's segment mesh: Linear, Arc, and Helix. These representations are what the `CurveParams` field in the `Segment` struct encodes, and they are what Phase 2 evaluates when integrating along segments to fill the impedance matrix.

The field-by-field mapping from NEC card parameters to these mathematical forms is specified in `docs/nec-import/card-reference.md`. This document is concerned only with the mathematics. This is where we hold it up to the light. 

---

## 2. Conventions

### 2.1 Coordinate System

Right-handed Cartesian coordinates (x, y, z) with z pointing up. The ground plane, when present, is at z = 0. All coordinates are in meters.

### 2.2 Parametric Variable

All curves are parameterized by t ∈ [0, 1], where t = 0 is the segment start and t = 1 is the segment end. For a wire discretized into N segments, segment k (0-indexed) corresponds to:

```
t ∈ [k/N, (k+1)/N]
```

Within a single segment, the local parameter τ ∈ [0, 1] maps to the global parameter as:

```
t = k/N + τ/N
```

Phase 2 integrates in τ over [0, 1] for each segment independently.

### 2.3 Quantities Phase 2 Requires

For every point on every segment, Phase 2 requires:

- **Position vector** r(τ) — location in 3D space
- **Tangent vector** r'(τ) = dr/dτ — direction and rate of change along the curve
- **Arc length element** ds = |r'(τ)| dτ — for converting parameter integrals to physical arc length
- **Unit tangent** t̂(τ) = r'(τ) / |r'(τ)| — direction of current flow

These four quantities must be computable at any τ, including at the Gauss-Legendre quadrature points used in Phase 2.

---

## 3. Linear Segment

### 3.1 Definition

A linear segment is a straight wire between two endpoints r_a and r_b.

### 3.2 Parametric Form

```
r(τ) = r_a + τ (r_b - r_a)
```

### 3.3 Derived Quantities

Tangent vector (constant along segment):
```
r'(τ) = r_b - r_a
```

Arc length element (constant):
```
ds = |r_b - r_a| dτ = L dτ
```

where L = |r_b - r_a| is the segment length.

Unit tangent (constant):
```
t̂ = (r_b - r_a) / L
```

Total arc length:
```
L = |r_b - r_a|
```

### 3.4 Notes

The linear segment is a degenerate case of the general parametric form. The tangent vector, arc length element, and unit tangent are all constant. This simplifies the Phase 2 integration significantly for straight-wire structures and provides a useful base case for numerical validation. We don't want straight-wire structures to be stupidly hard just because we can do curves. 

The linear segment is also the thin-wire kernel limit case. In the limit where all segments are linear and wire radii are small relative to segment length, Arcanum results must converge to classical thin-wire MoM results. See `docs/phase1-geometry/validation.md` cases V-LIN-001 through V-LIN-004.

---

## 4. Arc Segment

### 4.1 Definition

A circular arc segment is a section of a circle of radius R, centered at the origin, lying in the **XZ plane**. This follows the NEC GA card convention. Angles are measured from the positive x-axis toward the positive z-axis.

**Important:** The NEC GA card places arcs in the XZ plane, not the XY plane. This is a common source of confusion. Validation case V-ARC-001 in `validation.md` must use XZ plane coordinates. Any version of that case specifying y-coordinates as non-zero for a GA-sourced arc is incorrect and should be flagged as a documentation defect.

### 4.2 Parametric Form

Let θ₁ and θ₂ be the start and end angles in radians. For a segment that subtends the full arc from θ₁ to θ₂:

```
r(τ) = ( R cos(θ(τ)), 0, R sin(θ(τ)) )
```

where:
```
θ(τ) = θ₁ + τ(θ₂ - θ₁)
```

For segment k of N segments spanning the full arc:
```
θ₁ₖ = θ₁ + (k/N)(θ₂ - θ₁)
θ₂ₖ = θ₁ + ((k+1)/N)(θ₂ - θ₁)
```

and within segment k, τ ∈ [0, 1]:
```
θ(τ) = θ₁ₖ + τ(θ₂ₖ - θ₁ₖ)
```

### 4.3 Derived Quantities

Tangent vector:
```
r'(τ) = R(θ₂ₖ - θ₁ₖ) ( -sin(θ(τ)), 0, cos(θ(τ)) )
```

Arc length element (constant within a segment, since R and Δθ are constant):
```
ds = R |θ₂ₖ - θ₁ₖ| dτ
```

Unit tangent:
```
t̂(τ) = sign(θ₂ₖ - θ₁ₖ) · ( -sin(θ(τ)), 0, cos(θ(τ)) )
```

Total arc length of the full wire (all N segments):
```
L = R |θ₂ - θ₁|
```

where θ₂ - θ₁ is in radians.

### 4.4 Full Circle (Loop Antenna)

When θ₂ - θ₁ = 2π, the arc closes on itself. The end of segment N-1 coincides exactly with the start of segment 0 to double precision — this must hold analytically and numerically. 

Analytically:
```
r(τ=0, k=0) = ( R cos(θ₁), 0, R sin(θ₁) )
r(τ=1, k=N-1) = ( R cos(θ₁ + 2π), 0, R sin(θ₁ + 2π) )
             = ( R cos(θ₁), 0, R sin(θ₁) )   ✓
```

Any implementation that accumulates angle incrementally (adding Δθ per segment) rather than evaluating θ(τ) from the closed form will accumulate floating-point error and fail to close exactly. The closed form must be used. This is what validation case V-ARC-002 tests.

### 4.5 Rotation to Arbitrary Plane

The NEC GA card places arcs in the XZ plane. To obtain an arc in an arbitrary plane, the NEC GM card applies a rotation after GA. Phase 1 applies GM transformations to the final Cartesian coordinates of each segment endpoint — the parametric form above is evaluated first, then the rotation is applied as a standard 3D rotation matrix. The parametric form itself does not need to represent arbitrary orientations.

---

## 5. Helix Segment

### 5.1 Definition

A helical segment is a section of a uniform or tapered helix with its axis along the z-axis, starting at the origin. The helix is characterized by:

- **A₁** — radius at the start (z = 0 end)
- **A₂** — radius at the end (z = HL end)
- **S** — pitch: axial advance per turn (meters/turn)
- **HL** — total axial length of the helix (meters)
- **N_turns** — number of turns = HL / S

For a uniform helix, A₁ = A₂ = A.

### 5.2 Parametric Form

For the full helix, τ ∈ [0, 1]:

```
r(τ) = ( A(τ) cos(2π N_turns τ),
          A(τ) sin(2π N_turns τ),
          HL · τ )
```

where the radius varies linearly for a tapered helix:
```
A(τ) = A₁ + τ(A₂ - A₁)
```

For a uniform helix (A₁ = A₂ = A), A(τ) = A (constant).

For segment k of N segments, τ ∈ [k/N, (k+1)/N]. Within segment k, local parameter τ_k = k/N + σ/N for σ ∈ [0,1]:

```
r(σ) = ( A(τ_k(σ)) cos(2π N_turns τ_k(σ)),
          A(τ_k(σ)) sin(2π N_turns τ_k(σ)),
          HL · τ_k(σ) )
```

### 5.3 Derived Quantities — Uniform Helix

For the uniform case (A constant), differentiating with respect to σ:

```
r'(σ) = ( -(2π N_turns / N) A sin(2π N_turns τ_k(σ)),
            (2π N_turns / N) A cos(2π N_turns τ_k(σ)),
            HL / N )
```

Arc length element:
```
|r'(σ)| = (1/N) √( (2π N_turns A)² + HL² )
```

This is constant for a uniform helix — the arc length element does not vary with position along the helix. This is an important simplification for Phase 2 quadrature.

Unit tangent:
```
t̂(σ) = r'(σ) / |r'(σ)|
```

Total arc length of the full uniform helix:
```
L = √( (2π N_turns A)² + HL² )
```

This is the classic result: the helix unrolls into a right triangle with legs 2π N_turns A (circumferential) and HL (axial).

### 5.4 Derived Quantities — Tapered Helix

For A(τ) = A₁ + τ(A₂ - A₁), let A' = (A₂ - A₁) (constant derivative):

```
r'(σ) = ( (A'/N) cos(2π N_turns τ_k) - (2π N_turns A(τ_k)/N) sin(2π N_turns τ_k),
           (A'/N) sin(2π N_turns τ_k) + (2π N_turns A(τ_k)/N) cos(2π N_turns τ_k),
           HL/N )
```

Arc length element (no longer constant):
```
|r'(σ)| = (1/N) √( A'² + (2π N_turns A(τ_k))² + HL² )
```

The tapered helix arc length element varies with position. Phase 2 must evaluate it at each quadrature point, not assume it is constant.

### 5.5 Geometric Continuity Requirement

The critical requirement for helix discretization is that adjacent segment endpoints must be exactly coincident to double precision. This means segment endpoints must be evaluated from the closed-form parametric expression at t = k/N, not accumulated incrementally.

For segment k, the start point is:
```
r_start = r(k/N)
```

For segment k+1, the start point is:
```
r_start = r((k+1)/N)
```

These must be identical — evaluated from the same closed-form expression at the same parameter value. Any implementation that computes the end of segment k separately from the start of segment k+1 will produce a gap due to floating-point rounding. Use one evaluation, not two.

This requirement is validated by V-HEL-002, which checks that continuity gaps are below 1×10⁻¹² m across a 5-turn helix.

### 5.6 Handedness

A positive pitch S > 0 produces a right-handed helix (advances in +z as the angle increases in the standard counterclockwise direction). A negative pitch is not standard in NEC but should be handled gracefully — the implementation should not assume S > 0.

---

## 6. Ground Plane Image Geometry

When a PEC ground plane is present at z = 0, Phase 1 generates mirror image segments for all wire segments not lying in the ground plane. The image of a point (x, y, z) is (x, y, -z).

For each curve type, the image segment is obtained by negating the z-component of the parametric form:

**Linear image:**
```
r_image(τ) = ( r_x(τ), r_y(τ), -r_z(τ) )
```

**Arc image:**
The image of an arc in the XZ plane reflected through z = 0 is an arc in the XZ plane with angles negated:
```
θ_image = -θ
r_image(τ) = ( R cos(-θ(τ)), 0, -R sin(θ(τ)) )
           = ( R cos(θ(τ)), 0, -R sin(θ(τ)) )
```

**Helix image:**
The image of a right-handed helix is a left-handed helix with z negated:
```
r_image(τ) = ( A(τ) cos(2π N_turns τ),
                A(τ) sin(2π N_turns τ),
               -HL · τ )
```

Note that the image helix advances in the -z direction. The tangent vector z-component sign is reversed.

In all cases, image segments are appended to the segment list after all real segments and are flagged as images in the tag map.

---

## 7. Flag: Validation Document Discrepancy

**V-ARC-001 in `validation.md` contains an error.**

The expected endpoint coordinates for V-ARC-001 are written in the XY plane (z = 0 for all points), with the arc sweeping through non-zero y values. The NEC GA card places arcs in the XZ plane. The correct expected endpoints for V-ARC-001 are:

```
Segment 0 start: (0.5,  0.0,  0.0)       at θ =   0°
Segment 0 end:   (0.354, 0.0, 0.354)     at θ =  45°
Segment 1 end:   (0.0,   0.0, 0.5)       at θ =  90°
Segment 2 end:   (-0.354, 0.0, 0.354)    at θ = 135°
Segment 3 end:   (-0.5,  0.0, 0.0)       at θ = 180°
```

`validation.md` should be updated before Phase 1 implementation begins. This discrepancy is flagged here to ensure the math document and the validation document are reconciled during review.

---

## 8. References (our documents and most-cited books found through Wikipedia)

- `docs/phase1-geometry/design.md` — Segment struct definition, CurveType enum, junction handling
- `docs/phase1-geometry/validation.md` — Test cases that exercise these parametric forms
- `docs/nec-import/card-reference.md` — Mapping from NEC GW, GA, GH card fields to parametric parameters
- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — MoM foundations
- Balanis, C.A. — *Antenna Theory: Analysis and Design* — helix antenna geometry reference
- Kraus, J.D. — *Antennas* — uniform helix arc length formula and axial mode geometry

---

*Arcanum — Open Research Institute*
