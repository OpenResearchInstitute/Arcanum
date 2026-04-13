# Phase 3 — Matrix Solve Design

**Project:** Arcanum  
**Document:** `docs/phase3-matrix-solve/design.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document describes the architecture of Phase 3, the matrix solve. It specifies the data structures, assembly pipeline, solver interface, result extraction, and the interface to Phase 4.

Mathematical derivations are in `math.md`. Validation cases are in `validation.md`. This document is concerned with how the math is implemented.

---

## 2. Interface

### 2.1 Entry Point

```rust
pub fn solve(
    z_matrix: ZMatrix,
    sources: &[SourceDefinition],
    loads: &[LoadDefinition],
    mesh: &Mesh,
    frequency: f64,
    config: &SolveConfig,
) -> Result<SolveResult, SolveError>
```

- `z_matrix` — the impedance matrix from Phase 2. Phase 3 takes ownership; the working copy is modified in place for loads.
- `sources` — source definitions from the NEC parser (EX cards).
- `loads` — load definitions from the NEC parser (LD cards).
- `mesh` — read-only reference to the Phase 1 mesh. Required for per-segment geometry (arc lengths, radii) needed by wire conductivity loads.
- `frequency` — in Hz. No MHz conversion in Phase 3.
- `config` — solver configuration (residual threshold, condition number warning threshold).
- Returns `SolveResult` on success or `SolveError` on failure.

### 2.2 Phase Boundary

Phase 3 receives `ZMatrix` from Phase 2 and `SimulationInput` fields from the NEC parser. It returns `SolveResult` to Phase 4. It does not call Phase 1 or Phase 2 functions at runtime.

```
Phase 2 → ZMatrix
NEC parser → sources, loads
Phase 1 → Mesh (read-only reference)
                    ↓
            Phase 3: solve()
                    ↓
            SolveResult → Phase 4
```

---

## 3. Data Structures

### 3.1 SolveResult

```rust
pub struct SolveResult {
    pub currents: Vec<Complex<f64>>,      // [I] — one entry per segment
    pub input_impedances: Vec<InputImpedance>, // one per source
    pub residual: f64,                    // relative residual || ZI - V || / || V ||
    pub condition_number: f64,            // estimated condition number κ([Z])
    pub factorization: LuFactorization,   // preserved for multiple RHS (see Section 7)
}
```

### 3.2 InputImpedance

```rust
pub struct InputImpedance {
    pub segment_index: usize,     // global segment index of source
    pub z_in: Complex<f64>,       // Z_in = V_s / I[m_src]
    pub vswr_50: f64,             // VSWR relative to 50 Ω
    pub reflection_coeff: Complex<f64>, // Γ = (Z_in - Z_0) / (Z_in + Z_0)
}
```

### 3.3 SolveConfig

```rust
pub struct SolveConfig {
    pub residual_warning_threshold: f64,   // default 1e-8
    pub residual_error_threshold: f64,     // default 1e-4
    pub condition_warning_threshold: f64,  // default 1e10
    pub reference_impedance: f64,          // Z_0 for VSWR, default 50.0 Ω
}
```

### 3.4 SolveError

```rust
pub enum SolveError {
    SingularMatrix { condition_number: f64 },
    ResidualTooLarge { residual: f64 },
    NoSourcesDefined,
    SegmentIndexOutOfRange { tag: u32, segment: u32, global_index: usize },
}
```

---

## 4. Assembly Pipeline

Phase 3 proceeds through four sequential steps before invoking the solver.

### Step 1 — Copy ZMatrix

Phase 3 takes ownership of `ZMatrix` from Phase 2. A working copy is made immediately:

```rust
let mut z_working = z_matrix.clone();
```

The original is preserved in `z_matrix` for potential reuse (frequency sweep, multiple right-hand sides). Load modifications are applied to `z_working` only.

### Step 2 — Apply Loads

For each `LoadDefinition`, compute Z_load at the given frequency and add it to the appropriate diagonal elements of `z_working`.

Load application is sequential — loads are applied in the order they appear in the `loads` slice. For multiple loads on the same segment, their impedances accumulate on the diagonal:

```rust
for load in loads {
    let segment_indices = resolve_load_segments(load, mesh);
    for idx in segment_indices {
        let z_load = compute_load_impedance(load, frequency, mesh.segment(idx));
        z_working.add_to_diagonal(idx, z_load);
    }
}
```

`resolve_load_segments` returns the global segment indices for the range LDTAGF to LDTAGT on wire ITAG. For ITAG = 0 (all wires), it returns all segment indices.

`compute_load_impedance` dispatches on LDTYPE:
- LDTYPE 0 → series RLC (see `math.md` Section 4.2)
- LDTYPE 1 → parallel RLC (see `math.md` Section 4.3)
- LDTYPE 4 → distributed series RLC per unit length (Z_load scaled by segment arc length)
- LDTYPE 5 → wire conductivity (see `math.md` Section 4.4)

The ZLC = 0 guard (no capacitor → no 1/(jωC) term) must be implemented explicitly in `compute_load_impedance`. It is not safe to let the caller pass C = 0 and rely on floating-point behavior.

### Step 3 — Assemble Excitation Vector

Build the N×1 complex excitation vector [V]:

```rust
let mut v = vec![Complex::zero(); n_segments];

for source in sources {
    let global_idx = resolve_source_index(source, mesh);
    v[global_idx] += Complex::new(source.ex_real, source.ex_imag);
}
```

`resolve_source_index` maps (ITAG, ISEG) to a global segment index using the tag map from Phase 1:

```rust
fn resolve_source_index(source: &SourceDefinition, mesh: &Mesh) -> usize {
    let tag_entry = mesh.tag_map.get(source.itag)
        .expect("tag validated at parse time");
    tag_entry.start_index + (source.iseg as usize - 1)  // ISEG is 1-indexed
}
```

If no sources are defined, Phase 3 returns `SolveError::NoSourcesDefined` immediately — solving with a zero excitation vector produces a trivial zero solution with no physical meaning.

### Step 4 — Validate

Before invoking the solver:
- Confirm at least one non-zero entry in [V]
- Confirm `z_working` is square with dimension matching [V]
- Confirm all segment indices from source and load resolution are in range

These checks are defensive — they should have been caught at parse time. If they fail here, return `SolveError::SegmentIndexOutOfRange`.

---

## 5. Solver

### 5.1 LU Factorization via faer

```rust
let lu = z_working.lu_factorization();  // faer LU with partial pivoting
let currents = lu.solve(&v);            // forward + backward substitution
```

The `faer` crate provides `Mat<c64>::partial_piv_lu()` which returns a `PartialPivLu` struct. This struct holds the factorized form and supports:
- `.solve(&v)` — solve for one right-hand side
- `.solve_many(&V)` — solve for multiple right-hand sides simultaneously (used in Section 7)
- `.rcond()` — reciprocal condition number estimate

### 5.2 Residual Computation

After solving:

```rust
let residual_vec = z_working.mat_vec_mul(&currents) - &v;
let residual = residual_vec.l_inf_norm() / v.l_inf_norm();
```

If `residual > config.residual_error_threshold`, return `SolveError::ResidualTooLarge`.
If `residual > config.residual_warning_threshold`, add a warning to the result.

### 5.3 Condition Number

```rust
let rcond = lu.rcond();
let condition_number = 1.0 / rcond;
```

If `condition_number > config.condition_warning_threshold`, add a warning to the result identifying the condition number and suggesting model review.

### 5.4 Current Vector

The solved `currents: Vec<Complex<f64>>` has one entry per segment, in global segment index order matching the mesh. This is the primary output of Phase 3.

---

## 6. Input Impedance Extraction

For each source, extract the input impedance:

```rust
for source in sources {
    let idx = resolve_source_index(source, mesh);
    let v_s = Complex::new(source.ex_real, source.ex_imag);
    let i_src = currents[idx];
    let z_in = v_s / i_src;
    let gamma = (z_in - config.reference_impedance)
              / (z_in + config.reference_impedance);
    let vswr = (1.0 + gamma.norm()) / (1.0 - gamma.norm());
    // ...
}
```

Edge case: if `i_src` is zero or very small (open-circuit condition), Z_in → ∞. Report as infinity rather than NaN. This occurs for undriven elements in an array when the source is accidentally placed on the wrong segment — another reason to validate source placement carefully.

---

## 7. Multiple Right-Hand Sides

The `LuFactorization` is preserved in `SolveResult` to support multiple solves at the same frequency without re-factorizing. The caller pattern:

```rust
// First solve — factorizes [Z], solves for excitation A
let result_a = solve(z_matrix.clone(), &sources_a, &loads, &mesh, f, &config)?;

// Second solve — reuse factorization, new excitation B
let currents_b = result_a.factorization.solve_new_excitation(&sources_b, &mesh);
```

`solve_new_excitation` assembles a new [V] from the provided sources (no load modification needed — loads are already baked into the factorized matrix) and calls `lu.solve(&v_new)`.

This is the `ZGETRS` pattern described in `math.md` Section 5.3. It is most valuable for:
- Array pattern synthesis (same geometry, different excitation phases)
- Sensitivity analysis (perturbing source location)
- Bistatic RCS (multiple incident plane wave directions)

---

## 8. Frequency Sweep

For a frequency sweep, Phase 3 is called once per frequency. Each call receives a freshly filled `ZMatrix` from Phase 2 (the matrix changes at every frequency because k enters every element). The solve at each frequency is independent.

```rust
let results: Vec<SolveResult> = frequencies
    .iter()
    .zip(z_matrices.iter())
    .map(|(&f, z)| solve(z.clone(), &sources, &loads, &mesh, f, &config))
    .collect::<Result<Vec<_>, _>>()?;
```

Parallelism over frequencies is available to the caller via Rayon's `par_iter`. Individual solves are not parallelized internally — the O(N³) LU factorization does not parallelize well for small N. For large N (> 1000), `faer` uses multi-threaded BLAS internally, but this is transparent to Phase 3.

---

## 9. What Phase 3 Does Not Do

Phase 3 does not:

- **Compute radiation patterns.** That is Phase 4.
- **Compute near fields.** That is Phase 4.
- **Compute radiated power or efficiency.** That is Phase 4.
- **Re-fill the impedance matrix.** Phase 3 receives [Z] from Phase 2 and does not call Phase 2.
- **Re-parse NEC cards.** Source and load definitions are received as typed structs from the NEC parser.
- **Modify the mesh.** The `Mesh` is borrowed read-only.
- **Implement iterative solvers.** GMRES and other iterative methods are deferred from initial scope. Interested? Get in touch!

---

## 10. Open Items

1. **Distributed RLC load (LDTYPE = 4).** The per-unit-length formulation requires integrating Z_load(s) × ds along each segment, not simply multiplying by segment length. For curved segments, ds = |r'(τ)| dτ and the load is not uniformly distributed in parameter space. The implementation must use the parametric arc length, not the chord length. This is a minor but correct-by-construction concern. We ran into this in mechanical terms with the Dumbbell Antenna.

2. **Input power sign convention.** The formula P_in = (1/2) Re(V_s × I*[m_src]) assumes the e^{jωt} phasor convention. The implementation must be consistent with whatever convention `faer` uses for complex arithmetic. Verify with V-SOLVE-003: Re(Z_in) must be positive for a lossless antenna.

3. **VSWR for reactive source impedance.** The VSWR formula in Section 6 assumes a real reference impedance Z_0 = 50 Ω. For a complex source impedance (transmission line with non-negligible reactance), the formula changes. Deferred — real reference impedance covers the common case.

---

## 11. Anticipated Repository Layout

```
src/
├── matrix_solve/
│   ├── mod.rs              ← pub fn solve()
│   ├── assembly.rs         ← excitation vector assembly; load application
│   ├── load_models.rs      ← compute_load_impedance(); LDTYPE dispatch
│   ├── solver.rs           ← faer LU factorization; residual; condition number
│   ├── extraction.rs       ← input impedance; VSWR; reflection coefficient
│   ├── multi_rhs.rs        ← LuFactorization; solve_new_excitation()
│   ├── config.rs           ← SolveConfig
│   ├── results.rs          ← SolveResult; InputImpedance; SolveError
│   └── tests/
│       ├── assembly_tests.rs ← V-ASSM cases
│       ├── solve_tests.rs    ← V-SOLVE cases
│       └── cond_tests.rs     ← V-COND cases
```

---

## 12. References

- `docs/phase3-matrix-solve/math.md` — excitation models; load impedance formulas; LU method; Z_in extraction
- `docs/phase3-matrix-solve/validation.md` — test cases
- `docs/phase2-matrix-fill/design.md` — ZMatrix definition; faer storage layout
- `docs/phase4-postprocessing/design.md` — consumes SolveResult.currents
- Harrington, R.F. — *Field Computation by Moment Methods* (1968)
- faer crate documentation — PartialPivLu; solve; rcond

---

*Arcanum — Open Research Institute*
