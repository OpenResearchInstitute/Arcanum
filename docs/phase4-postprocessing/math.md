# Phase 4 — Post-Processing Mathematics

**Project:** Arcanum  
**Document:** `docs/phase4-postprocessing/math.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document derives the mathematical expressions used to compute all observable antenna parameters from the solved current vector [I]: far-field radiation patterns, directivity, gain, radiated power, radiation efficiency, and near fields. It provides the mathematical justification for the validation cases in `validation.md` and the implementation specification for `design.md`.

The current vector [I] from Phase 3 is the sole electromagnetic input to Phase 4. Everything computed here is an integral over that current distribution, using the same free-space Green's function as Phase 2 but evaluated at observation points external to the antenna structure rather than on the wire surface.

---

## 2. The Radiation Integral

### 2.1 Magnetic Vector Potential

For a wire antenna carrying segment currents I[m] (from Phase 3), the magnetic vector potential at an observation point r is:

```
A(r) = (μ₀/4π) Σ_{m=0}^{N-1} I[m] ∫_{Δm} t̂_m(s) G₀(r, r_m(s)) ds
```

where:
- The sum is over all N segments
- t̂_m(s) is the unit tangent to segment m at arc position s
- G₀(r, r') = e^{-jkR}/R is the free-space Green's function with R = |r - r_m(s)|
- The integral is over the arc length of segment m

This is the same integral structure as Phase 2's matrix fill, but with the observation point r located outside the antenna — at an arbitrary point in space rather than on the wire surface.

### 2.2 Far-Field Approximation

In the far field (r >> λ, r >> antenna dimensions), the Green's function simplifies. Let r = r r̂ where r = |r| and r̂ is the unit vector toward the observation point. Then:

```
R = |r - r_m(s)| ≈ r - r̂ · r_m(s)
```

This gives the far-field approximation:

```
G₀(r, r_m(s)) ≈ (e^{-jkr}/r) × e^{+jk r̂·r_m(s)}
```

The 1/R amplitude variation is approximated as 1/r (constant across the antenna aperture in the far field). The phase variation e^{+jk r̂·r_m(s)} retains the path length difference between segments — this is what produces the radiation pattern.

### 2.3 Far-Field Magnetic Vector Potential

Substituting the far-field approximation:

```
A(r) ≈ (μ₀/4π) × (e^{-jkr}/r) × Σ_{m=0}^{N-1} I[m] ∫_{Δm} t̂_m(s) e^{+jk r̂·r_m(s)} ds
```

The factor e^{-jkr}/r is common to all terms and represents propagation from the antenna to the observation point. The remaining sum is the **radiation vector** N(r̂):

```
N(r̂) = Σ_{m=0}^{N-1} I[m] ∫_{Δm} t̂_m(s) e^{+jk r̂·r_m(s)} ds
```

N(r̂) depends only on the observation direction r̂ — not on the distance r. This separability between direction and distance is the defining property of the far field.

---

## 3. Far-Field Electric and Magnetic Fields

### 3.1 Fields from Radiation Vector

In the far field, only the components of N transverse to the observation direction r̂ contribute to the radiated fields:

```
N_θ = N · θ̂    (θ-component of radiation vector)
N_φ = N · φ̂    (φ-component of radiation vector)
```

where θ̂ and φ̂ are the standard spherical coordinate unit vectors at the observation point.

The far-field electric field components are:

```
E_θ = -jωμ₀/(4π) × (e^{-jkr}/r) × N_θ
E_φ = -jωμ₀/(4π) × (e^{-jkr}/r) × N_φ
```

The far-field magnetic field components are:

```
H_θ = -E_φ / η₀
H_φ = +E_θ / η₀
```

where η₀ = √(μ₀/ε₀) ≈ 376.73 Ω is the impedance of free space.

### 3.2 Radiation Intensity

The time-averaged power radiated per unit solid angle (radiation intensity) in direction (θ, φ) is:

```
U(θ, φ) = (r²/2η₀) × (|E_θ|² + |E_φ|²)
         = (η₀k²)/(32π²) × (|N_θ|² + |N_φ|²)
```

Note that U is independent of r. Radiation intensity is a property of direction only, not distance. This confirms that the far-field approximation is correctly applied.

---

## 4. Directivity and Gain

### 4.1 Total Radiated Power

The total radiated power is the integral of radiation intensity over all solid angles:

```
P_rad = ∫∫ U(θ,φ) sin(θ) dθ dφ
      = ∫₀^π ∫₀^{2π} U(θ,φ) sin(θ) dθ dφ
```

Numerically, this integral is evaluated on a (θ,φ) grid with appropriate quadrature weights. The minimum grid resolution for accurate integration is discussed in `design.md`.

### 4.2 Directivity

The directivity D(θ,φ) is the radiation intensity normalized to the isotropic level (uniform radiation in all directions):

```
D(θ,φ) = U(θ,φ) / U_iso
        = 4π U(θ,φ) / P_rad
```

where U_iso = P_rad/(4π) is the radiation intensity of a hypothetical isotropic radiator with the same total radiated power.

The maximum directivity D_max = max_{θ,φ} D(θ,φ) is commonly expressed in dBi:

```
D_max [dBi] = 10 log₁₀(D_max)
```

**Normalization check:** Integrating D(θ,φ) over all solid angles must give 4π:

```
∫∫ D(θ,φ) sin(θ) dθ dφ = 4π
```

This is the validation case V-DIR-001 and must hold to within numerical integration accuracy.

### 4.3 Input Power and Gain

The input power delivered by the source is:

```
P_in = (1/2) Re(V_s × I*[m_src])
```

The radiation efficiency is:

```
η_rad = P_rad / P_in
```

For a lossless (PEC) antenna, η_rad = 1 exactly. For a lossy antenna, η_rad < 1.

The gain G(θ,φ) relates the radiation intensity to the total input power:

```
G(θ,φ) = 4π U(θ,φ) / P_in = η_rad × D(θ,φ)
```

The maximum gain G_max = η_rad × D_max.

---

## 5. Radiation Vector Numerical Evaluation

### 5.1 Segment Contribution

The radiation vector N(r̂) is a vector quantity. For each segment m, the contribution is:

```
N_m(r̂) = I[m] ∫_{Δm} t̂_m(τ) e^{+jk r̂·r_m(τ)} |r'_m(τ)| dτ
```

where the integral is over the local parameter τ ∈ [0,1], and |r'_m(τ)| is the arc length element from Phase 1 `math.md`.

### 5.2 Quadrature

The integrand is e^{+jk r̂·r_m(τ)} t̂_m(τ) |r'_m(τ)| — a smoothly varying complex vector. No singularity exists because the observation point r is in the far field, far from the antenna.

Standard Gauss-Legendre quadrature at order p = 8 is sufficient for electrically short segments (Δ < λ/5). For longer segments, the phase factor e^{+jk r̂·r_m(τ)} oscillates rapidly and higher quadrature order is needed. The criterion is:

```
p_required ≥ max(8, ceil(2 × k × Δ_m / π))
```

This ensures at least 2 quadrature points per half-period of the phase oscillation.

### 5.3 Coordinate System for N_θ and N_φ

The radiation vector N(r̂) is a 3D Cartesian vector. Its θ and φ components are obtained by projection:

```
N_θ = N · θ̂ = N_x cos(θ)cos(φ) + N_y cos(θ)sin(φ) - N_z sin(θ)
N_φ = N · φ̂ = -N_x sin(φ) + N_y cos(φ)
```

where (θ, φ) are the spherical coordinates of the observation direction r̂, and the Cartesian components of the spherical unit vectors are:

```
θ̂ = (cos(θ)cos(φ), cos(θ)sin(φ), -sin(θ))
φ̂ = (-sin(φ), cos(φ), 0)
```

---

## 6. Classical Antenna Pattern Formulas

These formulas provide analytic ground truth for the validation cases. They are derived from the radiation integral above in classical limiting cases.

### 6.1 Hertzian Dipole (Electrically Short)

For a short dipole of length Δ << λ carrying uniform current I₀ along the z-axis:

```
N_θ = I₀ Δ sin(θ)
N_φ = 0
```

Radiation intensity:

```
U(θ,φ) = (η₀k²|I₀Δ|²)/(32π²) × sin²(θ)
```

Directivity: D(θ,φ) = (3/2) sin²(θ), D_max = 3/2 = 1.5 (1.76 dBi).

This is the exact result for the Hertzian dipole and the limiting result for any electrically short dipole. Validation case V-DIR-003.

### 6.2 Half-Wave Dipole

For a half-wave dipole (length L = λ/2) along the z-axis carrying current I₀ at the feed:

```
N_θ = (2I₀/k) × cos(π/2 × cos(θ)) / sin(θ)
N_φ = 0
```

Normalized pattern function:

```
F(θ) = cos(π/2 × cos(θ)) / sin(θ)
```

Maximum directivity: D_max = 1.64 (2.15 dBi).

This is the basis for validation cases V-PAT-001 and V-DIR-002.

### 6.3 Pattern Null Condition

Both the Hertzian dipole and the half-wave dipole have pattern nulls on the dipole axis (θ = 0° and θ = 180°). In each case, N_θ → 0 as θ → 0 or θ → π:

- Hertzian dipole: N_θ ∝ sin(θ) → 0 as θ → 0
- Half-wave dipole: cos(π/2 × cos(0)) / sin(0) = cos(π/2)/0 = 0/0 → L'Hôpital: limit = 0

The numerical implementation must handle θ = 0° and θ = 180° as special cases to avoid 0/0 division. At these angles, the field is identically zero and must be returned as zero, not NaN.

---

## 7. Near-Field Computation

### 7.1 Near-Field Electric Field

The near-field electric field at observation point r (not in the far field) is computed from the full Green's function without the far-field approximation:

```
E(r) = -jωA(r) - ∇Φ(r)
```

where A(r) is the full magnetic vector potential (Section 2.1, without the far-field approximation) and Φ(r) is the electric scalar potential:

```
Φ(r) = 1/(4πε₀) Σ_{m=0}^{N-1} ∫_{Δm} ρ_m(s') G₀(r, r_m(s')) ds'
```

with the line charge density ρ_m(s') = -(1/jω) ∂I_m/∂s'.

For pulse basis functions (constant I[m] on each segment), ∂I/∂s' = 0 in the interior of each segment and I[m] changes abruptly at segment endpoints. The charge density is a sum of point charges at segment endpoints (as in Phase 2's T2 term).

### 7.2 Observation Points Outside the Wire

The near-field computation uses the same Green's function as Phase 2's matrix fill, but the observation point r is at an arbitrary location in space, not on the wire surface. For observation points more than one wire radius from any segment, the integrand is smooth and standard Gauss-Legendre quadrature at order p = 8 is sufficient.

For observation points very close to a wire segment (distance < 3 × wire radius), the integrand becomes nearly singular and higher-order quadrature is required. Arcanum warns when observation points are within 3 wire radii of any segment and uses adaptive quadrature in this region.

Observation points inside the wire (distance < wire radius) are not physically meaningful and should return an error, not silently produce wrong values.

### 7.3 Near-Field Magnetic Field

The near-field magnetic field is computed from the magnetic vector potential:

```
H(r) = (1/μ₀) ∇ × A(r)
```

For numerical implementation, the curl is evaluated by finite differences on the observation grid, or analytically by differentiating the Green's function under the integral sign (more accurate but more complex). The finite-difference approach is acceptable for initial implementation.

---

## 8. Ground Plane Contributions

When a PEC ground plane is present (GroundType::PEC), Phase 1 added image segments to the mesh. Phase 4, like Phase 2, uses the free-space Green's function over the complete mesh (real segments + image segments). The ground plane's effect on the radiation pattern is automatically included through the image currents. No special treatment is required in Phase 4. Cool!

The physical consequence: for an antenna over a PEC ground plane, the pattern is zero below the ground plane (z < 0) and enhanced above it due to constructive interference with the image antenna. This emerges naturally from the image segment contributions to the radiation vector N(r̂).

**Important:** Radiation patterns for antennas over ground planes are conventionally plotted only for the upper hemisphere (0° ≤ θ ≤ 90°). Phase 4 must support restricting pattern output to the upper hemisphere when a ground plane is present.

---

## 9. Radiated Power from Current Distribution

As an alternative to the far-field pattern integration, radiated power can be computed directly from the current distribution using the radiation resistance matrix. For completeness:

```
P_rad = (1/2) [I]^H [R_rad] [I]
```

where [R_rad] is the N×N radiation resistance matrix whose elements are:

```
R_rad[m,n] = Re(Z[m,n])   (real part of impedance matrix)
```

and [I]^H denotes the conjugate transpose of [I].

This provides a direct cross-check on the far-field integration result. Both methods must give the same P_rad — if they disagree, either the pattern integration has a numerical error or the impedance matrix has an error. In practice, the far-field integration method is preferred for pattern visualization and the impedance matrix method is used as a consistency check.

---

## 10. Axial-Mode Helix — Kraus Approximations

For reference and comparison against validation case V-HEL-001, the Kraus approximate formulas for an axial-mode helix antenna are:

**Gain:**
```
G ≈ 6.2 × N_turns × (C/λ)² × (S/λ)
```

where N_turns is the number of turns, C = 2πa is the helix circumference, and S is the turn spacing (pitch).

**Half-power beamwidth (HPBW):**
```
HPBW ≈ 52° / √(N_turns × (C/λ)² × (S/λ))
     ≈ 52° / √(N_turns × C/λ)   for C ≈ λ, S ≈ λ/4
```

**Input impedance (approximate):**
```
Z_in ≈ 140 × (C/λ)   Ω   (resistive, near-resonant)
```

These are engineering approximations valid for C ≈ λ (axial mode condition: 0.75λ < C < 1.33λ) and N_turns > 3. The numerical CMoM result should be within 20-30% of these values for a well-designed axial-mode helix.

---

## 11. References

- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — radiation vector formulation; Chapter 7
- Balanis, C.A. — *Antenna Theory: Analysis and Design* — far-field expressions; half-wave dipole pattern; directivity; Sections 2.4, 4.6
- Kraus, J.D. — *Antennas* — axial-mode helix gain and beamwidth; Chapter 7
- `docs/phase4-postprocessing/validation.md` — validation cases whose expected values are derived from this document
- `docs/phase1-geometry/math.md` — parametric forms r_m(τ), t̂_m(τ), |r'_m(τ)| used in radiation integrals
- `docs/phase3-matrix-solve/math.md` — current vector [I] and input power P_in

---

*Arcanum — Open Research Institute*
