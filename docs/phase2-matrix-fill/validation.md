# Phase 2 — Matrix Fill Validation

**Project:** Arcanum  
**Document:** `docs/phase2-matrix-fill/validation.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the validation cases for Phase 2, the matrix fill. Phase 2 computes every element Z[m,n] of the N×N impedance matrix. These validation cases certify the numerical integration engine in isolation, before any solver or antenna result is involved.

**Scope boundary:** Phase 2 validation tests individual matrix element values and matrix properties. It does not test input impedance, current distributions, or radiation patterns. Those require Phase 3 and Phase 4. When Phase 3 produces a wrong antenna impedance, a passing Phase 2 test suite means the fault is in the solve, not in the matrix fill. This is the certification value.

**What Phase 2 validation requires:** A `Mesh` struct (from Phase 1) and a frequency. What it produces under test: individual Z[m,n] values, or the full matrix, depending on the case.

---

## 2. Analytic Ground Truth Available

The following properties and limiting cases provide analytic ground truth for matrix element values without requiring a full antenna simulation:

- **Matrix symmetry** — Z[m,n] = Z[n,m] for all m,n (reciprocity, holds for any Galerkin formulation)
- **Diagonal real part sign** — Re(Z[m,m]) > 0 for all m (energy argument: self-resistance must be positive)
- **Thin-wire limit** — as wire radius a → 0 with segment length Δ fixed, exact kernel → thin-wire kernel result. The difference must vanish monotonically.
- **Far-field mutual coupling limit** — for widely separated segments (separation >> λ), Z[m,n] approaches a known free-space Green's function result
- **Quadrature convergence** — as quadrature order p increases, each Z[m,n] must converge monotonically to a stable value
- **Near-neighbor consistency** — Z[m,m+1] for adjacent segments must be handled by the near-singular path and produce values consistent with the far-field path in the limit of large separation

These properties hold regardless of the specific basis and testing function choices made in Phase 2 design. They are the correct validation targets at this stage.

---

## 3. Test Geometry Conventions

All Phase 2 test cases use geometries constructed programmatically from `MeshInput` structs and not from parsed `.nec` files. This keeps Phase 2 validation independent of the NEC parser.

Standard test parameters used across cases:

| Symbol | Value | Description |
|---|---|---|
| f | 300 MHz | Test frequency (λ = 1 m) |
| λ | 1.0 m | Free-space wavelength at f |
| Δ | λ/20 = 0.05 m | Standard segment length |
| a_thin | Δ/1000 = 5×10⁻⁵ m | Thin-wire regime (a << Δ) |
| a_mod | Δ/10 = 0.005 m | Moderate radius |
| a_thick | Δ/2 = 0.025 m | Thick wire — exact kernel advantage is largest here |

All geometries are in free space (no ground plane) unless otherwise specified.

---

## 4. Matrix Symmetry Cases (V-SYM)

### V-SYM-001 — Symmetry of Two-Segment Dipole Matrix

**Geometry:** Two collinear straight segments along the z-axis, each of length Δ = 0.05 m, radius a_mod. Total wire from z = -0.05 to z = 0.05. N = 2.

**Produces:** 2×2 impedance matrix.

**Pass criterion:**
```
|Z[0,1] - Z[1,0]| < ε_machine
```
where ε_machine is double-precision machine epsilon (~2.2×10⁻¹⁶). Symmetry should hold to full double precision, not just approximately.

---

### V-SYM-002 — Symmetry of 11-Segment Dipole Matrix

**Geometry:** 11 collinear straight segments, each Δ = 0.05 m, radius a_mod. N = 11.

**Produces:** 11×11 matrix.

**Pass criterion:** For all (m, n) pairs:
```
max |Z[m,n] - Z[n,m]| < ε_machine
```

This tests symmetry across the full matrix, including near-diagonal and far off-diagonal elements.

---

### V-SYM-003 — Symmetry of Mixed Geometry Matrix

**Geometry:** One straight segment (Δ = 0.05 m) and one arc segment (radius 0.05 m, subtending 90°, same arc length as straight segment), radius a_mod for both. N = 2. Segments are spatially separated so they are not adjacent.

**Pass criterion:** Z[0,1] = Z[1,0] to machine precision.

**Rationale:** Symmetry must hold across different segment types (linear and arc), not just between segments of the same type.

---

## 5. Diagonal Element Cases (V-DIAG)

### V-DIAG-001 — Diagonal Real Part is Positive

**Geometry:** Single straight segment, Δ = 0.05 m, radius a_mod. N = 1.

**Produces:** 1×1 matrix, a single complex value Z[0,0].

**Pass criterion:**
```
Re(Z[0,0]) > 0
```

**Rationale:** The real part of self-impedance represents radiation resistance plus ohmic loss. For a PEC segment it is pure radiation resistance, which must be positive by energy conservation.

---

### V-DIAG-002 — Diagonal Real Part Positive for All Segments

**Geometry:** 11-segment dipole as in V-SYM-002.

**Pass criterion:**
```
Re(Z[m,m]) > 0   for all m ∈ {0, ..., 10}
```

---

### V-DIAG-003 — Self-Impedance Scales with Frequency

**Geometry:** Single straight segment, Δ = λ/20 at each test frequency, radius a = Δ/100.

**Test frequencies:** 100 MHz, 300 MHz, 1000 MHz (segment length rescaled to λ/20 at each frequency to keep electrical length constant).

**Pass criterion:** The imaginary part of Z[0,0] (self-reactance) scales proportionally to frequency for electrically short segments. Specifically, Im(Z[0,0]) ∝ ω for fixed electrical length. A plot of Im(Z[0,0])/ω vs frequency should be flat within 1%.

**Rationale:** The inductive self-reactance of a short segment is ωL where L is the self-inductance — a geometric quantity independent of frequency. This is a dimensional consistency check on the integration.

---

## 6. Thin-Wire Limit Cases (V-THIN)

These cases validate that the exact kernel converges to the thin-wire approximation as a → 0. This is the fundamental correctness check on the exact kernel implementation.

### V-THIN-001 — Self-Impedance Thin-Wire Convergence

**Geometry:** Single straight segment, Δ = 0.05 m, z-axis. Radius varied.

**Test radii:** a = Δ×{0.1, 0.01, 0.001, 0.0001}

**Expected behavior:** As a → 0:
```
Z_exact_kernel[0,0] → Z_thin_wire[0,0]
```

The thin-wire self-impedance for a straight segment of length Δ in the limit a << Δ is:

```
Z_thin_wire ≈ jωμ₀Δ/(2π) × [ln(2Δ/a) - 1] + radiation term
```

(See `math.md` Section N for derivation.)

**Pass criterion:** The difference |Z_exact[0,0] - Z_thin[0,0]| decreases monotonically as a decreases through the test radii. At a = Δ/10000, the difference must be < 1% of |Z_thin_wire[0,0]|.

**Convergence plot required:** |Z_exact - Z_thin| vs a/Δ on a log-log scale. Must show the expected convergence slope.

---

### V-THIN-002 — Mutual Impedance Thin-Wire Convergence

**Geometry:** Two parallel straight segments of length Δ = 0.05 m, separated by perpendicular distance d = 5Δ = 0.25 m (far enough to be non-singular). Radius varied.

**Expected behavior:** For d >> a, the exact kernel mutual impedance Z[0,1] is insensitive to wire radius. The difference between exact kernel and thin-wire kernel should be negligible for all tested radii when segments are well-separated.

**Pass criterion:** |Z_exact[0,1] - Z_thin[0,1]| < 0.1% of |Z_thin[0,1]| for all tested radii. This confirms the exact kernel reduces to the thin-wire result for non-singular interactions.

---

### V-THIN-003 — Thick Wire Self-Impedance Divergence from Thin-Wire

**Geometry:** Single straight segment, Δ = 0.05 m, radius a = Δ/2 = 0.025 m (thick wire).

**Expected behavior:** At this radius, the thin-wire approximation is known to be inaccurate. The exact kernel result should differ meaningfully from the thin-wire result — this is the regime where exact kernel provides its advantage.

**Pass criterion:** |Z_exact[0,0] - Z_thin[0,0]| > 10% of |Z_thin[0,0]|. The exact and thin-wire results must diverge. If they agree at a = Δ/2, the exact kernel is not being evaluated correctly.

**Rationale:** This is a negative test — it verifies the exact kernel is actually doing something different from the thin-wire approximation in the regime where they should differ.

---

## 7. Quadrature Convergence Cases (V-QUAD)

These cases validate the Gauss-Legendre numerical integration by showing that Z[m,n] converges as quadrature order increases.

### V-QUAD-001 — Self-Impedance Quadrature Convergence

**Geometry:** Single straight segment, Δ = 0.05 m, radius a_mod.

**Test:** Compute Z[0,0] at quadrature orders p = {4, 8, 16, 32, 64}.

**Pass criterion:**
- Results converge monotonically toward a stable value Z_∞
- |Z(p=32) - Z(p=64)| / |Z(p=64)| < 1×10⁻⁸ (relative convergence to 8 significant figures)

**Convergence plot required:** |Z(p) - Z(p=64)| vs p on a semi-log scale. Should show exponential convergence characteristic of Gauss-Legendre quadrature on smooth integrands.

---

### V-QUAD-002 — Near-Neighbor Mutual Impedance Quadrature Convergence

**Geometry:** Two adjacent straight segments (end of segment 0 coincides with start of segment 1), each Δ = 0.05 m, radius a_mod.

**Test:** Compute Z[0,1] at quadrature orders p = {4, 8, 16, 32, 64}.

**Pass criterion:** Same convergence criterion as V-QUAD-001.

**Rationale:** Near-neighbor elements (adjacent segments sharing an endpoint) are near-singular — the integrand has a near-singularity when the source point approaches the shared endpoint. This case specifically tests the near-singular quadrature path. Slower convergence than V-QUAD-001 is expected and acceptable, but convergence must still be monotonic and reach 8 significant figures.

---

### V-QUAD-003 — Far Off-Diagonal Mutual Impedance Quadrature Convergence

**Geometry:** 11-segment dipole. Test element Z[0,10] — the interaction between the first and last segments.

**Test:** Compute Z[0,10] at quadrature orders p = {4, 8, 16, 32, 64}.

**Pass criterion:** Convergence to 8 significant figures at p = 32. Convergence for well-separated segments should be faster than near-neighbor cases.

**Rationale:** Far off-diagonal elements are smooth integrands — the integrand has no singularity. Gauss-Legendre should converge rapidly. Slow convergence here indicates a problem with the integration path, not the singularity handling.

---

## 8. Near-Singular Element Cases (V-NEAR)

These cases target the specific numerical challenge of self-impedance (singular) and near-neighbor (near-singular) elements, which require special treatment distinct from the smooth far-field quadrature.

### V-NEAR-001 — Self-Impedance Does Not Diverge

**Geometry:** Single segment, Δ = 0.05 m, radius a decreasing toward zero.

**Test:** Compute Z[0,0] at a = {Δ/10, Δ/100, Δ/1000}.

**Pass criterion:** Re(Z[0,0]) remains finite and positive at all radii. The imaginary part grows as ln(1/a) — this is expected and correct, not a divergence. The implementation must not produce NaN, Inf, or negative Re(Z[0,0]) at any tested radius.

---

### V-NEAR-002 — Near-Neighbor Element Does Not Diverge

**Geometry:** Two adjacent segments, each Δ = 0.05 m, radius a_thin.

**Pass criterion:** Z[0,1] is finite and non-NaN. Re(Z[0,1]) is finite and has the correct sign.

---

### V-NEAR-003 — Self vs Near-Neighbor Magnitude Ordering

**Geometry:** 11-segment dipole, radius a_mod. Consider the center segment (m = 5).

**Pass criterion:**
```
|Z[5,5]| > |Z[5,4]| > |Z[5,3]| > |Z[5,2]| > |Z[5,1]| > |Z[5,0]|
```

The self-impedance dominates, and off-diagonal elements decrease with separation. This is a physically required ordering — the coupling between two segments decreases as they move apart.

---

## 9. Arc Segment Matrix Cases (V-ARC)

### V-ARC-001 — Symmetry for Arc Segment Pair

**Geometry:** Two arc segments (each a short arc of a circle of radius 0.1 m, subtending 30°), spatially separated. N = 2.

**Pass criterion:** Z[0,1] = Z[1,0] to machine precision. Symmetry must hold for arc segments, not just linear segments.

---

### V-ARC-002 — Arc Self-Impedance Real Part Positive

**Geometry:** Single arc segment, radius 0.1 m, subtending 30°, wire radius a_mod.

**Pass criterion:** Re(Z[0,0]) > 0.

---

## 10. Helix Segment Matrix Cases (V-HEL)

### V-HEL-001 — Symmetry for Helix Segment Pair

**Geometry:** Two helix segments from a 1-turn helix (first and last segments of an 8-segment helix discretization). N = 2 (test matrix, not full helix).

**Pass criterion:** Z[0,1] = Z[1,0] to machine precision.

---

### V-HEL-002 — Helix Self-Impedance Real Part Positive

**Geometry:** Single helix segment, wire radius a_mod.

**Pass criterion:** Re(Z[0,0]) > 0.

---

## 11. Matrix Fill Performance Case (V-PERF)

### V-PERF-001 — Parallel Fill Produces Identical Results to Sequential Fill

**Geometry:** 11-segment dipole, radius a_mod.

**Test:** Compute the full 11×11 matrix twice — once with Rayon parallelism enabled, once with a single thread (parallelism disabled). Compare element by element.

**Pass criterion:**
```
max |Z_parallel[m,n] - Z_sequential[m,n]| < ε_machine
```

**Rationale:** Parallel matrix fill must be numerically identical to sequential fill. Any race condition or floating-point ordering dependency in the parallel implementation will show up here as a non-determinism or difference from the sequential result.

---

## 12. Validation Procedure

Each case is implemented as a Rust unit test in `src/matrix_fill/tests/`. Tests must:

1. Construct the input `Mesh` programmatically (not via the NEC parser)
2. Call the Phase 2 matrix fill entry point with the test `Mesh` and frequency
3. Assert the specific property or value specified, using the tolerance stated
4. For convergence cases, compute at all specified quadrature orders and assert monotonic convergence

Convergence plots for V-THIN-001, V-QUAD-001, V-QUAD-002, and V-QUAD-003 are required deliverables and must be committed alongside the implementation as figures in `docs/phase2-matrix-fill/figures/`.

---

## 13. References

- `docs/phase2-matrix-fill/math.md` — exact kernel integral formulation; thin-wire limit derivation; near-singular treatment
- `docs/phase2-matrix-fill/design.md` — matrix fill architecture; quadrature strategy; Rayon parallelism
- `docs/phase1-geometry/validation.md` — Phase 1 validation; Mesh construction used as input here
- Harrington, R.F. — *Field Computation by Moment Methods* (1968) — matrix element formulation
- Fikioris, G. — exact kernel references for thin-wire limit behavior

---

*Arcanum — Open Research Institute*
