# Phase 2 — Matrix Fill Design

**Project:** Arcanum  
**Document:** `docs/phase2-matrix-fill/design.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document describes the architecture of the Phase 2 matrix fill, the computation that produces the N×N impedance matrix [Z] from the segment mesh. It specifies data structures, the computation pipeline, element classification, quadrature strategy, parallelization, and the interface to Phase 3.

Mathematical derivations are in `math.md`. Validation cases are in `validation.md`. This document is concerned with how the math is implemented, not why it is correct.

---

## 2. Interface

### 2.1 Entry Point

```rust
pub fn fill_impedance_matrix(
    mesh: &Mesh,
    frequency: f64,
    config: &MatrixFillConfig,
) -> ZMatrix
```

- `mesh` — the segment mesh from Phase 1. Read-only. Phase 2 does not modify the mesh.
- `frequency` — simulation frequency in **Hz**. Phase 2 does not accept MHz. See `docs/nec-import/design.md` Section 4.7.
- `config` — quadrature and accuracy configuration (see Section 7).
- Returns a `ZMatrix` — the dense N×N complex impedance matrix.

### 2.2 Frequency Sweep

For a frequency sweep, the caller invokes `fill_impedance_matrix` once per frequency. There is no incremental update path (yet). The full matrix must be recomputed at each frequency because the wavenumber k appears in every matrix element.

```rust
// Caller pattern for frequency sweep
let matrices: Vec<ZMatrix> = frequencies
    .par_iter()
    .map(|&f| fill_impedance_matrix(&mesh, f, &config))
    .collect();
```

Parallelism over frequencies is available to the caller via Rayon. Parallelism within a single matrix fill is managed internally (Section 6).

### 2.3 Phase Boundary

Phase 2 receives `Mesh` from Phase 1 and returns `ZMatrix` to Phase 3. It does not call any Phase 1 function at runtime. The mesh is fully constructed before Phase 2 begins. It does not call Phase 3. The boundary is clean.

---

## 3. The ZMatrix

### 3.1 Storage

The impedance matrix is a dense N×N matrix of complex double-precision values:

```rust
pub struct ZMatrix {
    data: faer::Mat<faer::complex_native::c64>,
    n_segments: usize,
}
```

`faer::Mat<c64>` provides column-major dense storage, LU factorization (used in Phase 3), and BLAS-compatible layout. The choice of `faer` over `ndarray` or raw `Vec` is driven by Phase 3's requirement for direct LU factorization. Using the same library in both phases avoids a copy.

### 3.2 Memory Footprint

Each element is 16 bytes (two f64 values — real and imaginary). Memory requirement:

| N segments | Matrix size | Memory |
|---|---|---|
| 100 | 100×100 | 160 KB |
| 500 | 500×500 | 4 MB |
| 1,000 | 1,000×1,000 | 16 MB |
| 5,000 | 5,000×5,000 | 400 MB |

For ORI's primary use cases (ground station helices, yagis), N is expected to be in the range 50–500. The 5,000-segment case is the practical upper limit for direct LU solve on a workstation. It is not a hard limit in Phase 2.

### 3.3 Symmetry Exploitation

The impedance matrix is symmetric: Z[m,n] = Z[n,m] (see `math.md` Section 8). Only the upper triangle (m ≤ n) is computed. The lower triangle is filled by copying:

```
Z[n,m] ← Z[m,n]   for all m < n
```

This halves the number of integral evaluations. The copy is applied after all upper-triangle elements are computed, not during the parallel fill, to avoid any write-ordering complexity.

---

## 4. Element Classification

Before any integration begins, every (m,n) pair in the upper triangle is classified into one of three categories. Classification is O(N²) in accounting cost but trivial in computation. It is a "simple" index comparison. "Simple" matter of programming :D

### 4.1 Self Elements

```
m = n
```

Requires singularity extraction. One element per segment gives N total self elements.

### 4.2 Near-Neighbor Elements

```
|m - n| = 1   (adjacent segments in the mesh)
```

Requires near-singular quadrature. Two elements per segment (except at wire ends). This is approximately 2N total.

Note: "Adjacent in the mesh" means adjacent in the global segment index, which corresponds to adjacent along the same wire. Segments from different wires that happen to be geometrically close but are not sequentially adjacent in the mesh are treated as regular elements. This is a simplification. Future work may extend near-singular treatment to geometrically proximate non-sequential segments, if there's an advantage.

### 4.3 Regular Elements

```
|m - n| ≥ 2
```

Standard product Gauss-Legendre quadrature. The vast majority of elements, and O(N²) total.

---

## 5. Computation Pipeline

The matrix fill proceeds in four steps:

### Step 1 — Precompute Quadrature Tables

Gauss-Legendre nodes and weights are computed once at the start of the fill for each required quadrature order. These are stored in a `QuadratureTables` struct and shared (read-only) across all parallel workers. This is really neat.

```rust
struct QuadratureTables {
    gl_nodes_weights: HashMap<usize, (Vec<f64>, Vec<f64>)>,  // order → (nodes, weights)
    azimuthal_nodes_weights: (Vec<f64>, Vec<f64>),            // fixed order 16
}
```

Precomputed tables for orders {4, 8, 16, 32, 64} cover all expected use cases. The `gauss-quad` crate provides the nodes and weights.

### Step 2 — Classify Element Pairs

Build three lists:
- `self_elements: Vec<usize>` — diagonal indices m = n
- `near_neighbor_elements: Vec<(usize, usize)>` — pairs with |m-n| = 1, m < n
- `regular_elements: Vec<(usize, usize)>` — all other upper triangle pairs, m < n

### Step 3 — Parallel Fill

Fill all three categories in parallel using Rayon. Each category uses its own integration path:

```rust
// Regular elements — embarrassingly parallel
regular_elements
    .par_iter()
    .for_each(|&(m, n)| {
        let z = compute_regular(m, n, mesh, k, &quad_tables, &config);
        matrix.write(m, n, z);
    });

// Near-neighbor elements
near_neighbor_elements
    .par_iter()
    .for_each(|&(m, n)| {
        let z = compute_near_neighbor(m, n, mesh, k, &quad_tables, &config);
        matrix.write(m, n, z);
    });

// Self elements
self_elements
    .par_iter()
    .for_each(|&m| {
        let z = compute_self(m, mesh, k, &quad_tables, &config);
        matrix.write(m, m, z);
    });
```

The three categories can be filled in any order, including concurrently. Tthere are no data dependencies between elements. In practice, regular elements dominate the wall time and are filled first. This is the D&D cartographer job example. 

**Thread safety:** Each element write targets a unique (m,n) location. No two workers write to the same cell. `faer::Mat` supports this via raw pointer access. The `ZMatrix` wrapper must document this unsafety contract explicitly.

### Step 4 — Symmetry Copy

After all upper-triangle elements are computed:

```rust
for m in 0..n_segments {
    for n in (m+1)..n_segments {
        matrix.write(n, m, matrix.read(m, n));
    }
}
```

This is sequential and fast relative to the fill. It is not parallelized. The copy is O(N²) in count but trivially fast compared to the integration.

---

## 6. Integration Paths

### 6.1 Regular Element — compute_regular

Evaluates Z[m,n] for |m-n| ≥ 2 using a product Gauss-Legendre rule.

For each (s_i, s'_j) quadrature point pair:
1. Evaluate r_m(τ_i) and t̂_m(τ_i) from segment m's parametric form
2. Evaluate r_n(τ'_j), t̂_n(τ'_j), and endpoint positions r_n(s_n), r_n(s_{n+1})
3. Evaluate K_exact at (r_m(τ_i), r_n(τ'_j)) — the exact kernel (Section 6.3)
4. Accumulate T1 and T2 contributions (see `math.md` Section 11)
5. Apply weights and arc length elements |r'_m| and |r'_n|

Default quadrature order: p = 8 (64 function evaluations per element).

### 6.2 Near-Neighbor Element — compute_near_neighbor

Evaluates Z[m,n] for |m-n| = 1.

The integration is split: the smooth part uses standard Gauss-Legendre; the near-singular part near the shared endpoint uses an adaptive higher-order rule. The split point is determined by the ratio of the distance to the shared endpoint versus the wire radius.

**Implementation note:** The specific singularity extraction formula for the near-neighbor case is derived in the Fikioris references. This formula must be implemented exactly as derived. Approximations here produce systematic errors in the matrix that propagate through Phase 3 and produce wrong antenna impedances. This is the highest-risk implementation item in Phase 2. 

**Open item:** The near-neighbor extraction formula must be derived and written into `math.md` before `compute_near_neighbor` is implemented. See `math.md` Section 6 and Open Question 1.

### 6.3 Self Element — compute_self

Evaluates Z[m,m].

Singularity extraction is applied. The extracted singular part is integrated analytically; the smooth remainder is integrated with adaptive Gauss-Legendre.

Same open item as 6.2: the self-impedance extraction formula must be in `math.md` before implementation.

### 6.4 Exact Kernel Evaluation — evaluate_exact_kernel

```rust
fn evaluate_exact_kernel(
    r_obs: Vec3,          // observation point on segment m
    r_axis: Vec3,         // source point on segment n axis
    t_axis: Vec3,         // tangent at source point
    n_r: Vec3,            // radial normal at source point
    n_phi: Vec3,          // azimuthal normal at source point
    wire_radius: f64,     // radius of segment n
    k: f64,               // wavenumber
    az_nodes: &[f64],     // azimuthal quadrature nodes
    az_weights: &[f64],   // azimuthal quadrature weights
) -> Complex<f64>
```

For each azimuthal node φ_i:
1. Compute surface point: r_surf = r_axis + wire_radius × (n_r cos(φ_i) + n_phi sin(φ_i))
2. Compute R = |r_obs - r_surf|
3. Evaluate G₀ = e^(-jkR) / R
4. Accumulate: K += az_weights[i] × G₀

Return K / (2π) (the azimuthal average).

This function is called O(p²) times per matrix element for regular elements. It is the innermost loop of the entire computation and the primary target for micro-optimization after correctness is established.

---

## 7. Configuration

```rust
pub struct MatrixFillConfig {
    pub quadrature_order_regular: usize,      // default 8
    pub quadrature_order_near_singular: usize, // default 32 (adaptive starting point)
    pub quadrature_order_azimuthal: usize,    // default 16
    pub convergence_threshold: f64,           // default 1e-10 for adaptive
    pub near_singular_distance_ratio: f64,    // default 3.0 (multiples of wire radius)
}
```

Default values are conservative — they prioritize accuracy over speed. A `MatrixFillConfig::fast()` preset with relaxed thresholds is provided for exploratory use. Production simulations should use the default or tighter thresholds.

---

## 8. Crate Dependencies

| Crate | Version | Purpose |
|---|---|---|
| `faer` | latest | Dense matrix storage; shared with Phase 3 LU solver |
| `rayon` | latest | Data-parallel matrix fill |
| `gauss-quad` | latest | Gauss-Legendre nodes and weights |
| `nalgebra` | latest | Vec3 operations, rotation matrices for exact kernel normals |
| `num-complex` | latest | Complex<f64> arithmetic |

All crates must be pinned to specific versions in `Cargo.lock`. Floating latest versions are not acceptable for a numerical library where behavior changes between versions could produce silent accuracy regressions.

---

## 9. What Phase 2 Does Not Do

Phase 2 does not:

- **Assemble the excitation vector [V].** That is Phase 3. Phase 2 produces only [Z].
- **Solve the linear system.** That is Phase 3.
- **Know about sources or loads.** Phase 2 has no knowledge of EX or LD cards.
- **Modify the mesh.** The `Mesh` is borrowed immutably.
- **Handle lossy ground via Sommerfeld integrals.** Deferred from initial scope.
- **Cache matrices between calls.** Each call to `fill_impedance_matrix` computes from scratch. Caching is a future optimization.

---

## 10. Open Items

The following must be resolved before Phase 2 implementation begins:

1. **Self-impedance singularity extraction formula.** The specific analytic form of K_singular for the self-element case must be derived and written into `math.md` Section 5.2. This is the highest-risk item because an incorrect extraction formula produces systematically wrong diagonal elements that corrupt the entire solution.

2. **Near-neighbor extraction formula.** Same requirement for |m-n| = 1 elements. See `math.md` Section 6.

3. **Normal vector computation for exact kernel.** The vectors n̂_r and n̂_φ (perpendicular to the wire axis at each point) must be computed from the segment's tangent vector. For a helix segment, the Frenet-Serret frame is the natural choice. The computation of the principal normal and binormal from t̂ must be specified for each curve type. This is a `math.md` item.

4. **Unsafe write pattern review.** The parallel write pattern in Step 3 of Section 5 uses potentially unsafe raw pointer access to write to non-overlapping matrix locations. This must be reviewed by an experienced Rust contributor before implementation to confirm the "unsafety contract" is correctly stated and upheld. Risky part of the business. 

---

## 11. Repository Layout

```
src/
├── matrix_fill/
│   ├── mod.rs                ← pub fn fill_impedance_matrix()
│   ├── classify.rs           ← element classification
│   ├── quadrature.rs         ← QuadratureTables, GL node/weight precomputation
│   ├── exact_kernel.rs       ← evaluate_exact_kernel()
│   ├── regular.rs            ← compute_regular()
│   ├── near_neighbor.rs      ← compute_near_neighbor()
│   ├── self_element.rs       ← compute_self()
│   ├── config.rs             ← MatrixFillConfig
│   ├── zmatrix.rs            ← ZMatrix struct
│   └── tests/
│       ├── symmetry_tests.rs  ← V-SYM cases
│       ├── diag_tests.rs      ← V-DIAG cases
│       ├── thin_wire_tests.rs ← V-THIN cases
│       ├── quad_tests.rs      ← V-QUAD cases
│       ├── near_tests.rs      ← V-NEAR cases
│       └── perf_tests.rs      ← V-PERF cases
```

---

## 12. References

- `docs/phase2-matrix-fill/math.md` — EFIE formulation; exact kernel definition; Z[m,n] integral form
- `docs/phase2-matrix-fill/validation.md` — test cases
- `docs/phase1-geometry/math.md` — parametric forms r(τ), r'(τ), |r'(τ)|
- `docs/phase3-matrix-solve/design.md` — consumes ZMatrix; faer LU factorization
- Harrington, R.F. — *Field Computation by Moment Methods* (1968)
- Fikioris, G. & Wu, T.T. — exact kernel self-impedance extraction

---

*Arcanum — Open Research Institute*
