  # Phase 3 — Matrix Solve Mathematics

**Project:** Arcanum  
**Document:** `docs/phase3-matrix-solve/math.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the mathematics of Phase 3: the excitation vector assembly, load impedance models, the linear system and its solution, and the extraction of antenna parameters from the solved current vector. It provides the mathematical justification for the validation cases in `validation.md` and the implementation specification for `design.md`.

Phase 3 takes the impedance matrix [Z] from Phase 2 and the source/load definitions from the NEC parser, and produces the current vector [I]. Everything in this document works toward that result and what can be extracted from it.

---

## 2. The Linear System

### 2.1 Matrix Equation

At each frequency, the MoM formulation produces the linear system:

```
[Z] [I] = [V]
```

where:
- [Z] is the N×N complex impedance matrix from Phase 2
- [I] is the N×1 vector of unknown complex current amplitudes (one per segment)
- [V] is the N×1 complex excitation vector

The solution is:

```
[I] = [Z]⁻¹ [V]
```

[Z] is never explicitly inverted. the solution is obtained via LU factorization.

### 2.2 Superposition

The system is linear. For multiple simultaneous sources (multiple EX cards), the excitation vector [V] is the sum of the contributions from each source. The solution [I] is the current distribution for that combined excitation.

For the common case of a single driven element and one or more parasitic elements (Yagi, log-periodic), there is exactly one non-zero entry in [V] — the driven element source.

---

## 3. Excitation Vector Assembly

### 3.1 Delta-Gap Voltage Source (EXTYPE = 0)

The delta-gap model places a voltage source V_s across a vanishingly thin gap at the center of a segment. It is the standard NEC-2 source model and the model implemented in initial Arcanum.

The excitation vector entry for the m-th segment carrying a delta-gap source is:

```
V[m] = V_s
```

where V_s = EXREAL + j×EXIMAG from the EX card. All other entries are zero unless another source is present.

The delta-gap model is an approximation. It does not model the physical feed structure of the antenna. More sophisticated source models (the current slope discontinuity model, EXTYPE = 5) are deferred.

### 3.2 Global Index from Tag and Segment Number

The EX card identifies a source by wire tag ITAG and segment number ISEG (1-indexed from wire end 1). The corresponding global index into [V] is:

```
global_index = tag_map[ITAG].start_index + (ISEG - 1)
```

where `tag_map[ITAG].start_index` is the global segment index of the first segment of wire ITAG, established by Phase 1 during mesh construction.

This mapping must be exact. An off-by-one error here places the source on the wrong segment and produces a wrong current distribution that is not detectable by the residual check alone — [Z][I] = [V] will still be satisfied, but V[m] will be 1V at the wrong location. Bad stuff.

### 3.3 Multiple Sources

When multiple EX cards are present, each contributes one non-zero entry to [V]. If two EX cards reference the same segment (same ITAG, same ISEG), their voltages are summed. This is valid and represents two sources in series on the same segment gap. Physics!

---

## 4. Load Impedance Models

### 4.1 Load Application Principle

A load modifies the impedance matrix [Z] by adding the load impedance Z_load to the diagonal element corresponding to the loaded segment:

```
Z_modified[m,m] = Z[m,m] + Z_load(m, f)
```

This is correct for lumped series loads in the delta-gap model. The off-diagonal elements are not modified by lumped loads.

The modification is applied to a working copy of [Z]. The original matrix from Phase 2 is preserved. For frequency sweeps with the same geometry and loads but different frequencies, the Phase 2 matrix is reused and the load modification is reapplied at each frequency.

### 4.2 Series RLC Load (LDTYPE = 0)

The load impedance for a series R, L, C combination at angular frequency ω = 2πf:

```
Z_load = R + jωL + 1/(jωC)
       = R + j(ωL - 1/(ωC))
```

Field mapping from LD card:
- R (Ω) ← ZLR
- L (H) ← ZLI
- C (F) ← ZLC

If any component is absent (zero), its contribution is zero. For a purely resistive load, L = C = 0 and Z_load = R.

For C = 0 with L ≠ 0: Z_load = R + jωL (series RL, no capacitive term).
For L = 0 with C = 0: Z_load = R (purely resistive).

**Important:** ZLC = 0 must be treated as "no capacitor" (open circuit), not as an infinite capacitance (short circuit). Division by zero in the 1/(jωC) term must be explicitly guarded. This is a common implementation error.

### 4.3 Parallel RLC Load (LDTYPE = 1)

For a parallel R, L, C combination, the admittance is:

```
Y_load = 1/R + 1/(jωL) + jωC
```

The load impedance is:

```
Z_load = 1/Y_load = 1 / (1/R + 1/(jωL) + jωC)
```

Same zero-component guards apply: 1/R = 0 for R = 0 (short circuit in parallel); 1/(jωL) = 0 for L = 0 (open circuit in the inductor branch).

### 4.4 Wire Conductivity Load (LDTYPE = 5)

For a wire with finite conductivity σ (S/m), each segment has a distributed resistive loss. The per-segment series resistance is:

```
R_seg = Δ_m / (σ × 2π × a × δ_s)
```

where:
- Δ_m is the arc length of segment m (from Phase 1)
- a is the wire radius
- δ_s is the skin depth at frequency f:

```
δ_s = √(2 / (ωμσ)) = √(1 / (πfμσ))
```

For copper at 300 MHz: σ = 5.8×10⁷ S/m, μ = μ₀, δ_s ≈ 3.8 μm.

The physical interpretation: at RF frequencies, current flows only in a thin skin layer of depth δ_s at the wire surface. The effective cross-sectional area is the annulus of width δ_s at the surface: A_eff = 2πa × δ_s (for a >> δ_s, which holds for typical antenna wire at RF).

The diagonal entry for segment m becomes:

```
Z_modified[m,m] = Z[m,m] + R_seg(m)
```

Note that R_seg depends on segment arc length Δ_m (from Phase 1) and wire radius a — both are properties of the individual segment. For a tapered helix where A1 ≠ A2, each segment has a different radius and thus a different R_seg. The implementation must retrieve per-segment geometry from the mesh, not assume uniform wire properties.

---

## 5. LU Factorization

### 5.1 Method

The linear system [Z][I] = [V] is solved by LU factorization with partial pivoting:

```
P [Z] = L U
```

where:
- P is a permutation matrix (partial pivoting for numerical stability)
- L is unit lower triangular
- U is upper triangular

The solution proceeds in two steps:

**Forward substitution:** Solve L y = P V for y
**Backward substitution:** Solve U I = y for I

This is the standard LAPACK `ZGESV` algorithm, implemented in Arcanum via the `faer` crate.

### 5.2 Computational Cost

LU factorization of an N×N complex matrix costs O(N³) floating-point operations. For comparison:

| N | Approximate operations | Wall time (estimate, 1 core) |
|---|---|---|
| 100 | 2×10⁶ | < 1 ms |
| 500 | 2.5×10⁸ | ~ 100 ms |
| 1,000 | 2×10⁹ | ~ 1 s |
| 5,000 | 2.5×10¹¹ | ~ 15 min |

For ORI's primary use cases (N ≤ 500), the solve is fast relative to the matrix fill.

### 5.3 Frequency Sweep Efficiency

For a frequency sweep, [Z] must be refilled and refactored at each frequency (k changes, so every element of [Z] changes). There is no shortcut for the general case.

**Exception:** If the geometry is fixed and only the source location or amplitude changes between solves at the same frequency, the LU factorization can be reused. Phase 3 must preserve the factorized form of [Z] to support multiple right-hand sides at a single frequency:

```
[Z] = L U   (factorized once)
[I_a] = solve(L, U, V_a)   (source configuration a)
[I_b] = solve(L, U, V_b)   (source configuration b)
```

This is the standard LAPACK `ZGETRS` pattern. Factorize once, solve multiple times. We follow this design pattern. 

### 5.4 Residual Check

After solving, the residual is computed as a quality check:

```
r = || [Z][I] - [V] || / || [V] ||
```

where || · || is the L∞ norm (maximum absolute value). The relative residual r should be near machine epsilon (~10⁻¹⁵) for well-conditioned matrices. A residual above 10⁻⁸ warrants a warning; above 10⁻⁴ warrants an error.

The residual check requires one matrix-vector multiply after the solve. This is O(N²). This is cheap relative to the O(N³) factorization.

### 5.5 Condition Number Estimation

The condition number κ([Z]) = ||[Z]|| × ||[Z]⁻¹|| quantifies sensitivity of the solution to perturbations. Large κ means the solution [I] is sensitive to small errors in [Z] or [V].

The `faer` LU factorization provides a condition number estimate via the standard reciprocal condition number estimator (RCOND). The reported condition number is an estimate, not an exact value, but is reliable for practical purposes. 

Arcanum reports the condition number alongside the solve result. For ORI's antenna models, κ is expected to be in the range 10¹ to 10⁵ for well-designed models. Values above 10¹⁰ indicate a geometry or discretization problem that should be corrected in the model, not worked around in the solver.

---

## 6. Extracting Antenna Parameters from [I]

### 6.1 Input Impedance

The input impedance of the antenna at the source segment m_src is:

```
Z_in = V_s / I[m_src]
```

where V_s is the applied source voltage (from the EX card) and I[m_src] is the solved current at the source segment.

For a 1V source (the standard test condition), Z_in = 1 / I[m_src].

The input impedance is a complex number. Its real part is the input resistance (radiation resistance plus loss resistance); its imaginary part is the input reactance.

### 6.2 VSWR

For a source with reference impedance Z_0 (typically 50 Ω), the reflection coefficient is:

```
Γ = (Z_in - Z_0) / (Z_in + Z_0)
```

The VSWR is:

```
VSWR = (1 + |Γ|) / (1 - |Γ|)
```

VSWR = 1.0 indicates perfect match (Z_in = Z_0). VSWR = ∞ indicates total reflection (open or short circuit).

### 6.3 Radiated Power

The total radiated power is computed from the current distribution and the radiation resistance of each segment. However, the clean formulation requires the far-field integral from Phase 4. Phase 3 provides the current vector; Phase 4 computes radiated power and efficiency from it.

**Power available from Phase 3 alone:**

The input power delivered by the source is:

```
P_in = (1/2) Re(V_s × I*[m_src])
     = (1/2) |I[m_src]|² Re(Z_in)
```

This is the total power delivered to the antenna including both radiated power and ohmic losses. The split between radiation and loss requires Phase 4.

### 6.4 Current Distribution

The current vector [I] directly gives the complex current amplitude at every segment. This is the primary output of Phase 3 and the primary input to Phase 4.

The current at segment m is I[m] — a complex number representing amplitude and phase of the sinusoidal current at the center of segment m.

**Physical interpretation:** In the phasor convention used throughout (e^{jωt} time dependence suppressed), the instantaneous current at the center of segment m at time t is:

```
i_m(t) = Re(I[m] × e^{jωt})
       = |I[m]| cos(ωt + ∠I[m])
```

The phase ∠I[m] varies along the antenna structure and determines the radiation pattern through Phase 4.

---

## 7. Classical Validation Results

### 7.1 Half-Wave Dipole

The classical result for the input impedance of a thin half-wave dipole in free space is:

```
Z_in ≈ 73.1 + j42.5 Ω
```

This value is in the limit of a → 0 (infinitely thin wire) and exact half-wave length (L = λ/2). For finite wire radius and numerical discretization, the result varies slightly. The imaginary part (reactance) is most sensitive to these parameters — the resonant length for Im(Z_in) = 0 is slightly less than λ/2 for a real wire.

With 11 segments and a/λ = 0.001, the numerically computed Z_in should be within ±5 Ω of this classical value. This is the basis for validation case V-SOLVE-003.

### 7.2 Small Loop Radiation Resistance

For a small loop antenna with total circumference C << λ, the radiation resistance is:

```
R_rad = 20π²(C/λ)⁴
```

This is a fourth-power dependence on electrical size — small loops are very inefficient radiators. For C = λ/5 (= 0.2 m at 300 MHz), R_rad ≈ 0.32 Ω.

This formula is the basis for validation case V-SOLVE-005.

### 7.3 Reciprocity Theorem

For a passive linear electromagnetic system, the mutual impedance is symmetric:

```
Z_mn = Z_nm
```

where Z_mn is the open-circuit voltage induced at port n per unit current applied at port m. This follows from the reciprocity of the Green's function and holds exactly for the MoM formulation.

In the discrete system, this means the off-diagonal blocks of [Z] satisfy Z[m,n] = Z[n,m] (proved in Phase 2 `math.md` Section 8). After the solve, the current distribution satisfies the same reciprocity in terms of observable quantities. This is the basis for V-SOLVE-006.

---

## 8. References

- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — excitation models; load formulation; Chapter 5
- Balanis, C.A. — *Antenna Theory: Analysis and Design* — half-wave dipole impedance; small loop formula
- Burke & Poggio — *NEC-2 Method of Moments Code* (1981) — delta-gap source model; wire conductivity load
- `docs/phase2-matrix-fill/math.md` — [Z] matrix definition; symmetry proof
- `docs/phase3-matrix-solve/validation.md` — validation cases whose expected values are derived from this document
- `docs/phase4-postprocessing/math.md` — radiated power and efficiency computed from [I]

---

*Arcanum — Open Research Institute*
