# Phase 3 — Matrix Solve Validation

**Project:** Arcanum  
**Document:** `docs/phase3-matrix-solve/validation.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the validation cases for Phase 3, the matrix solve. Phase 3 has two distinct responsibilities, each with its own validation layer:

**Layer 1 — Assembly validation:** Does Phase 3 correctly build the excitation vector [V] from EX card data, and correctly apply LD card loads to [Z]? These are deterministic bookkeeping tests requiring no antenna theory.

**Layer 2 — Solve validation:** Does the LU factorization correctly solve [Z][I] = [V]? The residual check is the fundamental criterion. Antenna impedance results are the external ground truth.

These layers are tested independently. A passing Layer 1 with a failing Layer 2 means the assembly is correct but the solver is broken. A failing Layer 1 means antenna results are meaningless regardless of solver correctness.

**Scope boundary:** Phase 3 validation is the first point where full-pipeline results appear. Phase 2 must be passing before Phase 3 results are meaningful. A wrong matrix [Z] produces a wrong current [I] even with a perfect solver. Don't believe the cat when he says he hasn't been fed!

---

## 2. Layer 1 — Assembly Validation Cases (V-ASSM)

### V-ASSM-001 — Single Voltage Source, Center of Dipole

**Setup:** 11-segment dipole mesh (from Phase 1). Source definition: tag 1, segment 6, 1.0 + j0.0 V (center segment).

**Expected [V] vector:**
- V[5] = 1.0 + j0.0 (center segment, 0-indexed)
- All other entries = 0.0 + j0.0

**Pass criterion:** [V] has exactly one non-zero entry at index 5 with value 1+j0. All other entries are zero to machine precision.

---

### V-ASSM-002 — Source at Non-Center Segment

**Setup:** 11-segment dipole. Source: tag 1, segment 3, 0.5 + j0.5 V.

**Expected [V] vector:**
- V[2] = 0.5 + j0.5 (segment 3 is index 2 in 0-indexed)
- All other entries = 0.0 + j0.0

**Pass criterion:** Correct index mapping from NEC 1-indexed segment numbers to 0-indexed vector entries.

---

### V-ASSM-003 — Multiple Sources on Different Wires

**Setup:** Two-wire mesh (driven element + reflector, each 5 segments). Source on wire 1, segment 3. No source on wire 2.

**Expected [V] vector:**
- V[2] = 1.0 + j0.0 (wire 1, segment 3 → global index 2)
- V[5] through V[9] = 0.0 (wire 2, no source)

**Pass criterion:** Source correctly placed at global index corresponding to wire 1, segment 3. No source entries on wire 2.

---

### V-ASSM-004 — Resistive Load Modifies Diagonal

**Setup:** Single-segment mesh. Load definition: LDTYPE = 0 (series R), tag 1, segment 1, R = 50 Ω, L = 0, C = 0. Frequency = 300 MHz.

**Expected:** Z_modified[0,0] = Z_original[0,0] + 50.0 + j0.0

**Pass criterion:** The diagonal element is increased by exactly 50.0 Ω real. Off-diagonal elements are unchanged.

---

### V-ASSM-005 — Series RLC Load Modifies Diagonal at Frequency

**Setup:** Single-segment mesh. Load: LDTYPE = 0, R = 10 Ω, L = 10 nH, C = 10 pF. Frequency f = 300 MHz (ω = 2π × 3×10⁸).

**Expected load impedance:**
```
Z_load = R + jωL + 1/(jωC)
       = 10 + j(2π×3e8 × 10e-9) - j/(2π×3e8 × 10e-12)
       = 10 + j18.85 - j53.05
       = 10 - j34.2  Ω   (approximately)
```

**Pass criterion:** Z_modified[0,0] = Z_original[0,0] + Z_load, to 4 significant figures.

---

### V-ASSM-006 — Load Applied to All Segments of a Wire

**Setup:** 5-segment mesh, single wire. Load: LDTYPE = 5 (wire conductivity), tag 1, LDTAGF = 1, LDTAGT = 0 (all segments), conductivity σ = 3.72×10⁷ S/m (aluminum).

**Expected:** Resistive loss added to all 5 diagonal elements. The per-segment resistance is computed from σ, segment length, and wire radius. Each diagonal element Z[m,m] is increased by the segment resistance R_seg.

**Pass criterion:** All 5 diagonal elements increased by the same resistive term R_seg > 0. No off-diagonal modifications.

---

### V-ASSM-007 — Load Applied to Segment Range

**Setup:** 11-segment mesh. Load applied to segments 4 through 7 (LDTAGF = 4, LDTAGT = 7, 1-indexed).

**Expected:** Diagonal elements Z[3,3] through Z[6,6] (0-indexed) modified. All other diagonal elements unchanged.

**Pass criterion:** Exactly 4 diagonal elements modified, correct range, no off-diagonal changes.

---

## 3. Layer 2 — Solve Validation Cases (V-SOLVE)

### V-SOLVE-001 — Residual Check, 2×2 Known Matrix

**Setup:** Construct a 2×2 complex impedance matrix analytically:
```
Z = [100+j50    10-j5 ]
    [10-j5      80+j30]
```
Excitation vector: V = [1+j0, 0+j0].

**Solve for:** [I] = Z⁻¹ [V]

**Expected residual:**
```
r = |[Z][I] - [V]| / |[V]| < 1×10⁻¹²
```

**Pass criterion:** Relative residual below 1×10⁻¹². This tests the LU factorization on a small, known, well-conditioned matrix before any antenna geometry is involved.

---

### V-SOLVE-002 — Residual Check, 11-Segment Dipole

**Setup:** Full pipeline — Phase 1 mesh, Phase 2 matrix fill, Phase 3 solve. 11-segment half-wave dipole at 300 MHz, center source 1V.

**Expected residual:**
```
r = max_m |([Z][I])_m - V_m| / |V_m_max| < 1×10⁻¹⁰
```

**Pass criterion:** Residual below 1×10⁻¹⁰. This is the fundamental correctness criterion for the solve on a real antenna matrix. It does not require knowing the correct antenna impedance — it only requires that [Z][I] = [V] is satisfied.

---

### V-SOLVE-003 — Input Impedance, Half-Wave Dipole

**Setup:** 11-segment half-wave dipole, center-fed, at 300 MHz (λ = 1 m, total length 0.5 m), wire radius 0.001 m. Source: 1V at center segment.

**Input impedance from solve:**
```
Z_in = V_source / I_source = 1.0 / I[5]
```

**Expected result:**
```
Re(Z_in) ≈ 73 Ω    (radiation resistance)
Im(Z_in) ≈ 42.5 Ω  (reactance, inductive for half-wave)
```

**Tolerance:** ±5 Ω on both real and imaginary parts. This tolerance accounts for discretization error with 11 segments — convergence to the classical result improves with increasing N.

**Convergence requirement:** As N increases from 11 to 21 to 41 segments, Z_in must converge monotonically toward 73 + j42.5 Ω. A convergence plot of Z_in vs N is a required deliverable.

**Note:** This is the first validation case requiring a known physical antenna result. It depends on Phase 2 being correct. A failure here after Phase 2 passes V-SOLVE-002 indicates a problem in Phase 3 assembly, not in Phase 2.

---

### V-SOLVE-004 — Input Impedance, Short Dipole

**Setup:** 11-segment dipole, length = λ/10 = 0.1 m, center-fed, at 300 MHz, wire radius 0.001 m.

**Expected result:**
```
Re(Z_in) ≈ 2 Ω      (low radiation resistance for electrically short antenna)
Im(Z_in) ≈ -1500 Ω  (strongly capacitive — large negative reactance)
```

**Tolerance:** ±20% on real part (low absolute value makes percentage tolerance appropriate); ±200 Ω on imaginary part.

**Rationale:** A short dipole is a demanding test because the radiation resistance is very small relative to the reactance. The condition number of [Z] is high for short dipoles. If the solve handles the half-wave case but fails here, the LU factorization likely has a conditioning issue.

---

### V-SOLVE-005 — Input Impedance, Small Loop

**Setup:** 8-segment circular loop, radius = λ/(10π) ≈ 0.0318 m, wire radius 0.001 m, at 300 MHz. Source at segment 1.

**Expected result (small loop, C << λ):**
```
Re(Z_in) ≈ 20π²(C/λ)⁴   (radiation resistance, very small)
Im(Z_in) > 0              (inductive, positive reactance)
```

where C = 2π × 0.0318 ≈ 0.2 m is the loop circumference.

**Tolerance:** Re(Z_in) must be positive and within a factor of 2 of the analytic small-loop formula. Im(Z_in) must be positive (inductive).

**Rationale:** This is the first test case that specifically exercises the CMoM advantage over NEC-2. The circular loop with curved arc segments should give more accurate results than an equivalent polygon approximation. The analytic small-loop result is a classical formula — see `math.md`.

---

### V-SOLVE-006 — Two-Port Reciprocity Check

**Setup:** Two parallel half-wave dipoles, each 11 segments, separated by λ/4. Solve twice:
- Run A: source on dipole 1, measure induced voltage on dipole 2
- Run B: source on dipole 2, measure induced voltage on dipole 1

**Expected result (reciprocity):**
```
Z_12 = V_2 / I_1  (Run A) = V_1 / I_2 (Run B) = Z_21
```

**Pass criterion:**
```
|Z_12 - Z_21| / |Z_12| < 1×10⁻⁸
```

**Rationale:** Reciprocity is a fundamental property of passive linear electromagnetic systems. If the matrix is correctly filled and the solve is correct, reciprocity must hold. A failure indicates either a non-symmetric matrix (Phase 2 bug) or an asymmetric modification to [Z] in Phase 3 (load assembly bug).

This test is quite possibly the most elegant test in Phase 3. It's a pure mathematical property of passive linear systems that requires no analytic antenna formula. You just solve the same structure twice with source and receiver swapped. If Z_12 ≠ Z_21, something is wrong in either the matrix fill or the load assembly. It's a self-consistency check that doesn't depend on knowing the right answer.

---

### V-SOLVE-007 — Resistive Load Reduces Reflected Power

**Setup:** 11-segment dipole, center-fed, at 300 MHz. Two solves:
- Run A: no load
- Run B: 50 Ω resistive load at center segment (in series with source)

**Expected result:**
```
Re(Z_in_B) > Re(Z_in_A)   (added resistance increases input resistance)
```

**Pass criterion:** The real part of input impedance increases when a resistive load is added. The increase must equal the added load resistance to within numerical precision:

```
Re(Z_in_B) - Re(Z_in_A) ≈ 50 Ω   (within ±1 Ω)
```

---

### V-SOLVE-008 — Frequency Sweep Consistency

**Setup:** 11-segment dipole, center-fed. Solve at 5 frequencies: 250, 275, 300, 325, 350 MHz.

**Expected result:**
- Z_in varies smoothly and continuously with frequency
- At 300 MHz, Z_in matches V-SOLVE-003
- The reactance crosses zero near resonance (near 300 MHz for a half-wave dipole at that frequency)

**Pass criterion:** No discontinuities or NaN values across the sweep. Reactance sign change (zero crossing) present between 250 and 350 MHz.

---

## 4. Condition Number Monitoring Cases (V-COND)

### V-COND-001 — Condition Number Reported for Well-Conditioned Matrix

**Setup:** 11-segment half-wave dipole. The condition number of [Z] is computed as part of the LU factorization.

**Expected:** Condition number κ([Z]) is finite, reported in the solve output, and in the range 10¹ to 10⁴ for a well-designed half-wave dipole model.

**Pass criterion:** Condition number is computed without error and reported. A value outside the expected range generates a warning (not an error).

---

### V-COND-002 — Ill-Conditioned Matrix Warning

**Setup:** Construct a pathologically ill-conditioned 3×3 matrix with κ > 10¹² analytically.

**Expected:** Phase 3 solve completes but emits a warning identifying the high condition number and noting that results may be inaccurate.

**Pass criterion:** Warning is emitted. Solve does not panic or produce NaN.

---

## 5. Validation Procedure

Each assembly case (V-ASSM) must be implemented as a Rust unit test that:
1. Constructs a mesh and source/load definitions programmatically
2. Calls the Phase 3 assembly entry point
3. Asserts the specific vector or matrix modification expected

Each solve case (V-SOLVE) must be implemented as a Rust integration test that:
1. Runs the full Phase 1 → Phase 2 → Phase 3 pipeline
2. Computes the residual and/or input impedance from the result
3. Asserts against the specified tolerances

The convergence plot for V-SOLVE-003 (Z_in vs N for the half-wave dipole) is a required deliverable and must be committed to `docs/phase3-matrix-solve/figures/`.

---

## 6. References

- `docs/phase3-matrix-solve/math.md` — excitation assembly, load models, input impedance formula
- `docs/phase3-matrix-solve/design.md` — LU solver architecture, excitation vector assembly
- `docs/phase2-matrix-fill/validation.md` — Phase 2 must pass before Phase 3 antenna results are meaningful
- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — reciprocity theorem
- Balanis, C.A. — *Antenna Theory* — half-wave dipole impedance reference value

---

*Arcanum — Open Research Institute*
