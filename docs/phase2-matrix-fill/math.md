# Phase 2 — Matrix Fill Mathematics

**Project:** Arcanum  
**Document:** `docs/phase2-matrix-fill/math.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document derives the mathematical form of every element Z[m,n] of the MoM impedance matrix, defines the exact kernel, and specifies the numerical quadrature strategy used to evaluate the integrals. It provides the mathematical justification for the validation cases in `validation.md` and the implementation specification for `design.md`. This is from Harrington.

The central quantity is the impedance matrix element Z[m,n]. Everything in this document works toward a precise, implementable expression for that quantity.

---

## 2. The Electric Field Integral Equation

### 2.1 Problem Statement

Consider a perfectly electrically conducting (PEC) wire antenna in free space. An incident electric field **E**_inc illuminates the antenna, inducing a surface current density **J** that radiates a scattered field **E**_scat. On the wire surface, the boundary condition requires the total tangential electric field to vanish:

```
(E_inc + E_scat)_tan = 0   on the wire surface
```

This is the fundamental boundary condition from which everything follows.

### 2.2 Scattered Field from Wire Currents

The scattered electric field is expressed through the magnetic vector potential **A** and the electric scalar potential Φ:

```
E_scat = -jω A - ∇Φ
```

For a wire antenna carrying a line current I(s) along its arc length coordinate s, the potentials are:

```
A(r) = (μ₀/4π) ∫_C I(s') t̂(s') G(r, r(s')) ds'

Φ(r) = 1/(4πε₀) ∫_C ρ(s') G(r, r(s')) ds'
```

where:
- C is the wire curve
- t̂(s') is the unit tangent to the wire at s'
- G(r, r') is the Green's function kernel (defined in Section 4)
- ρ(s') is the line charge density, related to current by continuity: ρ = -(1/jω) ∂I/∂s'

### 2.3 The EFIE

Substituting into the boundary condition and taking the tangential component along the wire axis, the Electric Field Integral Equation (EFIE) is:

```
-E_inc_tan(s) = jωμ₀/(4π) ∫_C I(s') [t̂(s)·t̂(s')] G(s,s') ds'
              + 1/(jωε₀4π) ∫_C ∂I(s')/∂s' · ∂G(s,s')/∂s ds'
```

This is a Fredholm integral equation of the first kind in the unknown I(s). It cannot be solved analytically for arbitrary wire geometries. This is why MoM is needed.

---

## 3. Method of Moments Discretization

### 3.1 Basis Function Expansion

The wire is divided into N segments by Phase 1. The current I(s) is approximated as a superposition of basis functions f_n(s):

```
I(s) ≈ Σ_{n=0}^{N-1} I_n f_n(s)
```

where I_n are the unknown complex current amplitudes — the entries of the solution vector [I].

Arcanum uses **pulse basis functions**: f_n(s) = 1 for s ∈ segment n, zero elsewhere. This is the simplest choice and is consistent with NEC-2. Higher-order basis functions (triangular, sinusoidal) are a future extension.

### 3.2 Galerkin Testing

Substituting the basis expansion into the EFIE and applying **Galerkin testing** — weighting with the same basis functions. This produces a system of N linear equations. The m-th equation is obtained by integrating the EFIE against f_m(s) over segment m:

```
∫_{Δm} f_m(s) × [EFIE] ds = 0   for m = 0, ..., N-1
```

This produces the matrix equation:

```
[Z] [I] = [V]
```

where:
- [Z] is the N×N impedance matrix
- [I] is the N×1 vector of unknown current amplitudes
- [V] is the N×1 excitation vector (assembled in Phase 3)

### 3.3 The Impedance Matrix Element

The element Z[m,n] — the interaction between testing segment m and basis segment n — is:

```
Z[m,n] = jωμ₀/(4π) ∫_{Δm} ∫_{Δn} [t̂_m(s)·t̂_n(s')] K(r_m(s), r_n(s')) ds ds'
        - 1/(jωε₀4π) ∫_{Δm} ∫_{Δn} [∂/∂s ∂/∂s' K(r_m(s), r_n(s'))] ds ds'
```

where K is the kernel (Green's function), defined in Section 4.

The two terms have physical interpretations:
- **First term** — magnetic vector potential coupling: how the current on segment n contributes to the tangential vector potential along segment m. Depends on the dot product of the tangent vectors.
- **Second term** — electric scalar potential coupling: how the charge on segment n (via ∂I/∂s') contributes to the tangential electric field along segment m.

Both integrals are double integrals over two one-dimensional arc lengths. The arcs of segments m and n respectively. 

### 3.4 Simplified Form for Pulse Basis

For pulse basis functions, ∂I/∂s' = 0 everywhere except at segment endpoints, where it is a delta function. Integrating by parts, the second term becomes a sum over endpoint contributions. For segment n with endpoints at s = s_n (start) and s = s_{n+1} (end):

```
∂I_n/∂s' → I_n [δ(s' - s_{n+1}) - δ(s' - s_n)]
```

The impedance matrix element becomes:

```
Z[m,n] = jωμ₀/(4π) ∫_{Δm} ∫_{Δn} [t̂_m·t̂_n] K ds ds'
        - 1/(jωε₀4π) ∫_{Δm} [K(r_m(s), r_n(s_{n+1})) - K(r_m(s), r_n(s_n))] ds
```

This is the form that is implemented in the matrix fill. See `design.md` for the specific quadrature used for each term.

---

## 4. The Green's Function Kernel

### 4.1 Free-Space Green's Function

The free-space Green's function is:

```
G₀(r, r') = e^(-jkR) / R
```

where:
- R = |r - r'| is the distance between field point r and source point r'
- k = 2π/λ = ω√(μ₀ε₀) = ω/c is the free-space wavenumber

### 4.2 The Thin-Wire Kernel

In the thin-wire approximation, both the source and observation points are placed on the wire axis. For a wire segment at position r_axis(s'), the kernel is evaluated at the axis:

```
K_thin(s, s') = G₀(r_m(s), r_n_axis(s')) = e^(-jk R_axis) / R_axis
```

where R_axis = |r_m(s) - r_n_axis(s')| is the axis-to-axis distance.

**Limitation:** When segments m and n are adjacent (sharing an endpoint) or identical (m = n), R_axis → 0 and the kernel diverges. NEC-2 handles this with an approximation that breaks down for thick wires and bent geometries.

### 4.3 The Exact Kernel

The exact kernel replaces the axis-point source with the average of the Green's function over the full circumference of the wire surface at the source point:

```
K_exact(s, s') = 1/(2π) ∫₀^{2π} G₀(r_m(s), r_n_surf(s', φ)) dφ
```

where r_n_surf(s', φ) is a point on the surface of segment n at arc position s' and azimuthal angle φ around the wire axis:

```
r_n_surf(s', φ) = r_n_axis(s') + a [n̂_r(s') cos(φ) + n̂_φ(s') sin(φ)]
```

where a is the wire radius, and n̂_r, n̂_φ are unit vectors in the plane perpendicular to the wire axis at s'.

**Why this matters:** For m = n (self-impedance), R_surf = |r_m(s) - r_n_surf(s', φ)| has a minimum value of a (the wire radius) rather than zero. The singularity is regularized by the wire's physical extent. For thick wires and curved geometries, this regularization is not just a numerical convenience, it is physically correct.

### 4.4 Exact Kernel — Evaluation

The azimuthal integral in K_exact is evaluated numerically. For the implementation, the outer double integral over segments m and n is evaluated with Gauss-Legendre quadrature (Section 6), and the inner azimuthal integral is evaluated with a fixed-order quadrature (typically 8–16 points suffices for most wire radii).

For segment self-interaction (m = n), the azimuthal integral is bounded. The integrand does not diverge. The minimum value of R_surf is a, achieved when s = s' and φ aligns the surface point with the observation point.

---

## 5. Self-Impedance — The Singular Case (m = n)

### 5.1 Nature of the Singularity

For the self-impedance element Z[m,m], the source and observation points are on the same segment. As s → s', R_axis → 0 in the thin-wire kernel, producing a logarithmic singularity. This is an integrable singularity (∫₀ ln(r) dr is finite) but requires careful numerical handling.

With the exact kernel, R_surf ≥ a > 0 — the singularity does not occur. However, when a is small, R_surf is small for s ≈ s' and the integrand is sharply peaked. Standard fixed-order Gauss-Legendre quadrature undersamples the peak and produces inaccurate results. This is the near-singular case.

### 5.2 Treatment — Singularity Extraction

The standard approach is singularity extraction. Split the self-impedance integral into a singular (or near-singular) part with a known analytic form, and a smooth remainder that is well-suited to Gauss-Legendre:

```
K_exact(s, s') = [K_exact(s, s') - K_singular(s, s')] + K_singular(s, s')
                 ─────────────────────────────────────   ─────────────────
                          smooth remainder               known analytic part
```

The analytic part K_singular is chosen to match the singular behavior of K_exact as s → s'. The smooth remainder is then integrated with standard Gauss-Legendre at modest quadrature order. The analytic part is integrated exactly.

The specific form of K_singular and its analytic integral are derived in terms of the segment geometry and wire radius. See the Fikioris references for the exact kernel self-impedance extraction.

### 5.3 Thin-Wire Limit of Self-Impedance

In the thin-wire limit (a → 0), the exact kernel self-impedance converges to the known thin-wire self-impedance formula. For a straight segment of length Δ:

```
Z_self_thin ≈ jωμ₀Δ/(2π) × [ln(2Δ/a) - 1] + R_rad
```

where R_rad is the radiation resistance contribution (real part), which is small for electrically short segments (Δ << λ).

The logarithmic dependence on a/Δ confirms that the imaginary part of self-impedance grows without bound as a → 0 — this is correct and expected behavior, not a divergence to be suppressed. The validation case V-THIN-001 verifies this convergence.

---

## 6. Near-Neighbor Elements — The Near-Singular Case

### 6.1 Adjacent Segments

For adjacent segments m and m+1 (sharing one endpoint), the minimum axis-to-axis distance R_axis can become very small as the integration variables s and s' approach the shared endpoint. With the exact kernel, R_surf ≥ a, so there is no true singularity — but for small a, the integrand is again sharply peaked near the shared endpoint.

The same singularity extraction approach applies. Identify the near-singular behavior analytically, subtract it, integrate the remainder with standard quadrature, add the analytic contribution.

### 6.2 Extent of Near-Neighbor Treatment

The near-singular treatment is applied to the self-segment (m = n) and the two immediate neighbors (m = n±1). Elements more than one segment apart are treated as smooth integrands with standard Gauss-Legendre quadrature.

For most practical antenna models (Δ ≈ λ/20, a ≈ Δ/50), the integrand is smooth for |m-n| ≥ 2 and standard quadrature at order 8–16 achieves 10 or more significant figures.

---

## 7. Gauss-Legendre Quadrature

### 7.1 Standard Elements (|m-n| ≥ 2)

For non-singular, non-near-singular matrix elements, the double integral over segments m and n is evaluated with a product Gauss-Legendre rule:

```
∫_{Δm} ∫_{Δn} f(s, s') ds ds' ≈ Σ_i Σ_j w_i w_j f(s_i, s_j) |ds/dτ| |ds'/dτ'|
```

where s_i, w_i are the Gauss-Legendre nodes and weights on [-1, 1], mapped to the segment arc length interval. The arc length elements |ds/dτ| = |r'_m(τ)| are computed from the parametric representations in `docs/phase1-geometry/math.md`.

Quadrature order p = 8 (8 points per dimension, 64 function evaluations per element) is the default. This achieves approximately 10 significant figures for smooth integrands.

### 7.2 Near-Singular Elements (|m-n| ≤ 1)

For self and near-neighbor elements, after singularity extraction the smooth remainder is integrated with an adaptive Gauss-Legendre rule. The quadrature order is increased until the contribution from successive orders changes by less than a convergence threshold ε_quad.

The default convergence threshold is ε_quad = 1×10⁻¹⁰ (10 significant figures). This threshold can be relaxed for performance or tightened for higher-accuracy simulations.

### 7.3 Azimuthal Integration for Exact Kernel

The azimuthal integral in K_exact is evaluated with a fixed-order Gauss-Legendre rule on [0, 2π]. For most wire radii (a ≤ Δ/2), order 16 (16 azimuthal points) achieves 8 significant figures in K_exact. For a/Δ > 0.5 (very thick wires), higher azimuthal order may be required.

---

## 8. Matrix Symmetry — Proof

The impedance matrix [Z] is symmetric: Z[m,n] = Z[n,m]. This follows from the reciprocity of the free-space Green's function:

```
G₀(r, r') = G₀(r', r)   (G₀ depends only on |r - r'|)
```

Since the EFIE operator is self-adjoint under Galerkin testing with a reciprocal kernel, Z[m,n] = Z[n,m] exactly in the continuous limit. In the discrete implementation, symmetry should hold to machine precision — any deviation indicates a programming error in the quadrature or parametric evaluation.

This is the basis for the validation cases V-SYM-001 through V-SYM-003.

---

## 9. Free-Space vs Ground Plane

The derivation above assumes free space. When a PEC ground plane is present at z = 0, the Green's function is modified by the method of images:

```
G_ground(r, r') = G₀(r, r') + G₀(r, r'_image)
```

where r'_image is the image of r' reflected through z = 0.

Phase 1 handles this by adding image segments to the mesh. Phase 2 uses the free-space kernel G₀ — it does not need to know about the ground plane directly, because the image segments are real segments in the mesh as far as Phase 2 is concerned. This is the geometric ground plane treatment described in `docs/phase1-geometry/design.md` Section 6.

For lossy ground (Sommerfeld/Wait model), the Green's function is modified by Sommerfeld integrals that depend on ground conductivity and permittivity. This is a Phase 2 extension, not in initial scope.

---

## 10. Wavenumber and Physical Constants

All integrals are evaluated at a specific frequency f (Hz). The relevant physical constants and derived quantities:

```
c  = 2.99792458×10⁸ m/s     (speed of light)
μ₀ = 4π×10⁻⁷ H/m            (permeability of free space)
ε₀ = 1/(μ₀c²) F/m           (permittivity of free space)
η₀ = √(μ₀/ε₀) ≈ 376.73 Ω   (impedance of free space)

ω  = 2πf                     (angular frequency, rad/s)
k  = ω/c = 2π/λ              (free-space wavenumber, rad/m)
λ  = c/f                     (free-space wavelength, m)
```

Frequencies are received from Phase 2's input in Hz (conversion from MHz was applied in the NEC parser semantic stage. See `docs/nec-import/design.md` Section 4.7). Phase 2 must not contain any MHz-to-Hz conversion logic.

---

## 11. Summary: Implementable Form of Z[m,n]

Collecting the results above, the complete implementable form for a non-singular element Z[m,n] (|m-n| ≥ 2) is:

```
Z[m,n] = jωμ₀/(4π) × T1[m,n]  -  1/(jωε₀4π) × T2[m,n]
```

where:

```
T1[m,n] = ∫_{Δm} ∫_{Δn} [t̂_m(s)·t̂_n(s')] K_exact(s,s') ds ds'

T2[m,n] = ∫_{Δm} [K_exact(s, r_n(s_{n+1})) - K_exact(s, r_n(s_n))] ds
```

Both integrals are evaluated with the product Gauss-Legendre rule described in Section 7.1. The arc length elements ds = |r'(τ)| dτ are computed from the parametric forms in `docs/phase1-geometry/math.md` Sections 3–5.

For self and near-neighbor elements (|m-n| ≤ 1), singularity extraction is applied before quadrature as described in Sections 5 and 6.

---

## 12. Open Questions

1. **Basis function order:** Pulse basis is specified here for initial implementation. Sinusoidal basis functions (as in Hallén's equation formulation) are more accurate for longer segments. This is a Phase 2 extension and should be tracked as a GitHub issue or at least a Discussion. It's not really kicked down the road in a can, but it kind of is.

2. **Lossy ground Sommerfeld integrals:** The Wait model for lossy ground modifies the Green's function and requires Sommerfeld integral evaluation. This is mathematically well-defined but computationally expensive. Deferred from initial scope because I'm not sure how to grapple it in. 

3. **Azimuthal quadrature order:** The value of 16 points for the exact kernel azimuthal integral is a recommendation based on the literature. Validation case V-QUAD-001 should be used to verify this is sufficient for the targeted accuracy. This might blow up. 

---

## 13. References

- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — MoM formulation; Sections 3-4 directly relevant, cannot be overstated
- Fikioris, G. & Wu, T.T. — "On the Application of Numerical Methods to Hallén's Equation" — exact kernel self-impedance extraction essentially lifted
- Vande Ginste, D. et al. — conformal MoM basis function formulations for curved wires essentially lifted
- Burke & Poggio — *NEC-2 Method of Moments Code* (1981) — thin-wire kernel formulation (the baseline we improve upon!)
- `docs/phase1-geometry/math.md` — parametric forms r(τ), r'(τ), |r'(τ)| used in arc length elements
- `docs/phase2-matrix-fill/validation.md` — validation cases whose expected values are derived from this document

---

*Arcanum — Open Research Institute*
