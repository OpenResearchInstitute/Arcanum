# Phase 2 (Matrix Fill) Implementation Plan

## Context

Phase 1 (Geometry) is complete and all Phase 2 design blockers are resolved. Phase 2 computes the N×N complex impedance matrix Z[m,n] from the segment mesh using the exact cylindrical kernel formulation with adaptive Gauss-Legendre quadrature. The `arcanum-matrix-fill` crate exists as a stub. All math, algorithms, validation cases, and module structure are specified in `docs/phase2-matrix-fill/`.

---

## Prerequisite: Add Curve Evaluation to Phase 1

**Problem:** `CurveParams` has no `evaluate(σ)`, `tangent(σ)`, or `speed(σ)` methods. Phase 2 needs `r(σ)`, `r'(σ)`, `|r'(σ)|` at every quadrature point. Furthermore, after GM transforms, `ArcParams` and `HelixParams` only have Cartesian endpoints updated — the rotation matrix is discarded, making arbitrary-σ evaluation impossible for transformed arcs/helices.

**Solution:** Add `rotation: Matrix3<f64>` and `center: Vector3<f64>` fields to `ArcParams` and `HelixParams`. Implement `evaluate()`, `tangent()`, `speed()`, `arc_length()` on `CurveParams`.

### Files to modify

**`crates/arcanum-geometry/src/mesh.rs`**
- Add `rotation: Matrix3<f64>`, `center: Vector3<f64>` to `ArcParams` and `HelixParams`
- Add `impl CurveParams` with:
  - `evaluate(σ) → Vector3<f64>` — position r(σ) for σ ∈ [0,1]
  - `tangent(σ) → Vector3<f64>` — dr/dσ (unnormalized)
  - `speed(σ) → f64` — |dr/dσ|
  - `arc_length() → f64` — total segment arc length
- Linear: `r(σ) = start + σ*(end - start)`, tangent = `end - start` (constant)
- Arc: `r(σ) = rotation * (R cos θ(σ), 0, R sin θ(σ))ᵀ + center` where `θ(σ) = θ1 + σ(θ2 - θ1)`
- Helix: `r(σ) = rotation * (A(τ) cos(2πNτ), A(τ) sin(2πNτ), HL·τ)ᵀ + center` where `τ = (seg_idx + σ)/n_segs`

**`crates/arcanum-geometry/src/discretize.rs`**
- Set `rotation: Matrix3::identity()`, `center: Vector3::zeros()` in `discretize_arc()` and `discretize_helix()`

**`crates/arcanum-geometry/src/transforms.rs`**
- `transform_segment()` Arc/Helix: compose `p.rotation = rot * p.rotation`, `p.center = rot * p.center + trans`
- `scale_segment()` Arc/Helix: scale `p.center *= s`

**`crates/arcanum-geometry/src/images.rs`**
- `reflect_z()` Arc/Helix: compose z-reflection `diag(1,1,-1)` with rotation, negate `center.z`

**`crates/arcanum-geometry/src/tests/` — new `curve_eval.rs`**
- `evaluate(0.0)` == `start()`, `evaluate(1.0)` == `end()` for all curve types
- Arc midpoint at σ=0.5 matches known value
- Helix midpoint at σ=0.5 matches known value
- Evaluation correct after GM rotation+translation
- `speed()` for linear = segment length
- `arc_length()` for arc = R × |θ2 - θ1|

**Verify:** `cargo test -p arcanum-geometry` — all existing + new tests pass

---

## Phase 2 Implementation Steps

### Step 1: Add dependencies to Cargo.toml

**`crates/arcanum-matrix-fill/Cargo.toml`** — add:
- `faer = { workspace = true }` — dense matrix storage (Mat\<c64\>)
- `num-complex = "0.4"` — Complex\<f64\> arithmetic
- `gauss-quad = "0.2"` — Gauss-Legendre nodes/weights

**Verify:** `cargo check -p arcanum-matrix-fill`

### Step 2: config.rs — MatrixFillConfig

```rust
pub struct MatrixFillConfig {
    pub quadrature_order_regular: usize,        // 8
    pub quadrature_order_near_singular: usize,  // 32
    pub quadrature_order_azimuthal: usize,      // 16
    pub convergence_threshold: f64,             // 1e-10
    pub near_singular_distance_ratio: f64,      // 3.0
}
```

With `Default` impl and `MatrixFillConfig::fast()` preset.

### Step 3: zmatrix.rs — ZMatrix wrapper

Wraps `faer::Mat<c64>`. Methods: `new(n)`, `read(m,n)`, `write(m,n,val)` (unsafe interior for parallel writes), `n_segments()`, `data()`, `into_inner()`. Document the safety contract for parallel writes.

### Step 4: constants.rs — Physical constants

`C_LIGHT`, `MU_0`, `EPS_0`, `wavenumber(freq) → f64`. Unit test: `MU_0 * EPS_0 * C_LIGHT²` ≈ 1.0.

### Step 5: quadrature.rs — GL table precomputation

`QuadratureTables` stores precomputed GL nodes/weights for orders {4,8,16,32,64} and azimuthal nodes/weights on [0,2π]. Uses `gauss_quad::GaussLegendre`. Unit test: weights sum to 2.0, integral of x² over [-1,1] = 2/3.

### Step 6: classify.rs — Element classification

Classify upper-triangle (m,n) pairs into self (m=n), near-neighbor (n=m+1), regular (n>m+1). Unit test for N=4: 4 self + 3 near + 3 regular = 10 upper-triangle entries.

### Step 7: exact_kernel.rs — Exact kernel + perpendicular frame

- `perpendicular_frame(t̂) → (n̂_r, n̂_φ)` — minimum-component Gram-Schmidt
- `evaluate_exact_kernel(r_obs, r_axis, t̂, wire_radius, k, az_nodes, az_weights) → c64` — azimuthal integration of G₀ over wire surface

Unit tests: frame orthogonality, kernel approaches G₀(axis) for large separation.

### Step 8: regular.rs — Regular element integration

`compute_regular(m, n, mesh, k, quad_tables, config) → c64`

Product GL quadrature at order `quadrature_order_regular` for T1 and T2 terms. Each quadrature point calls `evaluate_exact_kernel`. Combines: `Z = jωμ₀/(4π) T1 - 1/(jωε₀·4π) T2`.

### Step 9: self_element.rs — Self-element integration

`compute_self(m, mesh, k, quad_tables, config) → c64`

Singularity extraction for T1: analytic `I_near = 2[Δ sinh⁻¹(Δ/a) - √(a²+Δ²) + a]` plus GL on smooth remainder `K_exact - K_near`. T2 via standard GL at endpoints. Uses `quadrature_order_near_singular`.

### Step 10: near_neighbor.rs — Near-neighbor integration

`compute_near_neighbor(m, n, mesh, k, quad_tables, config) → c64`

Local coordinates ε, ε' from shared endpoint. Analytic `I_near` for collinear case. Smooth remainder via GL at `quadrature_order_near_singular`. Dot product `t̂_m · t̂_n` accounts for bend angle. Cross-wire junctions use same formula (Option A).

### Step 11: lib.rs — Entry point

`fill_impedance_matrix(mesh: &Mesh, frequency: f64, config: &MatrixFillConfig) → ZMatrix`

Pipeline: precompute tables → classify → parallel fill (Rayon `par_iter` for each category) → symmetry copy → return.

### Step 12: Validation tests

`crates/arcanum-matrix-fill/src/tests/` with test helpers to build meshes programmatically.

| Suite | Tests | What's checked |
|-------|-------|---------------|
| symmetry.rs | V-SYM-001..003 | Z[m,n] = Z[n,m] to machine ε |
| diag.rs | V-DIAG-001..003 | Re(Z[m,m]) > 0, freq scaling |
| thin_wire.rs | V-THIN-001..003 | Convergence to thin-wire formula |
| quad.rs | V-QUAD-001..003 | Monotonic convergence, 8 sig figs at p=32 |
| near.rs | V-NEAR-001..003 | No NaN/Inf, magnitude ordering |
| arc.rs | V-ARC-001..002 | Arc symmetry, positive real diagonal |
| helix.rs | V-HEL-001..002 | Helix symmetry, positive real diagonal |
| perf.rs | V-PERF-001 | Parallel == sequential to machine ε |

### Step 13: PyO3 bindings

**`crates/arcanum-py/src/lib.rs`** — add `PyMatrixFillConfig`, `PyZMatrix`, `fill_impedance_matrix()` function. Follow existing wrapper pattern.

**`crates/arcanum-py/Cargo.toml`** — add `arcanum-matrix-fill` dependency.

### Step 14: Python integration test

**`tests/matrix_fill/test_fill.py`** — smoke test: parse NEC deck → build mesh → fill matrix → check symmetry and dimensions.

---

## Key Risks and Open Questions

All risks investigated and resolved as of 2026-06-07.

### Risk 1: faer c64 type compatibility — RESOLVED

`faer::complex_native::c64` is not `num_complex::Complex<f64>`. Need to verify:
- How to construct `c64` values (likely `c64::new(re, im)`)
- Whether arithmetic operations (+, *, exp) are available on `c64` or whether we compute in `Complex<f64>` and convert at the write boundary
- Whether `faer::Mat<c64>` supports element-level read/write with the API we need

**Resolution (confirmed):** The faer 0.19 `c64` API is fully suitable:
- Construction: `c64::new(re, im)`, `c64::cis(phase)`, `c64::from_polar(r, θ)`
- All arithmetic ops implemented: `Add`, `Sub`, `Mul`, `Div`, `Neg` with both `c64` and `f64` operands, plus compound assignment variants
- Full math function set: `.exp()`, `.sin()`, `.cos()`, `.sqrt()`, `.ln()`, `.conj()`, `.norm()`, etc.
- Conversion: `From<Complex<f64>>` and `.to_num_complex()` available but not needed — all kernel math can be done directly in `c64`
- Element access on `Mat<c64>`: `.read(row, col)` / `.write(row, col, val)` (bounds-checked), `.read_unchecked()` / `.write_unchecked()`, and raw pointer access via `.ptr_at_mut(row, col)`
- Same memory layout as `Complex<f64>` (two contiguous `f64` fields: `pub re: f64`, `pub im: f64`)

### Risk 2: Unsafe parallel writes to faer::Mat — RESOLVED

ZMatrix needs concurrent writes from Rayon threads to non-overlapping matrix cells. `faer::Mat` is not designed for concurrent mutation.

**Options:**
- (A) Use raw pointer arithmetic on `Mat::as_ptr_mut()` — requires `unsafe`, well-documented invariant
- (B) Use `UnsafeCell` wrapper or custom allocation — more code, same safety argument
- (C) Allocate per-thread buffers and merge — safe but doubles memory and adds copy overhead

**Resolution (confirmed):** Option A is feasible and sound. faer 0.19 provides:
- `mat.ptr_at_mut(row, col)` — returns `*mut c64` to a specific cell
- `mat.col_stride()` — offset between columns (column-major layout)
- `mat.row_stride()` — always 1 for owned `Mat`

Safety argument for concurrent writes via raw pointers:
1. `Mat` is allocated before and not resized/dropped during the parallel region
2. Each `(row, col)` pair is written by exactly one Rayon thread (non-overlapping)
3. No concurrent reads occur during the fill phase
4. `c64` is `Copy + Send` — writing is a plain 16-byte memcpy

Implementation should encapsulate the unsafe in a single helper and document the invariants with a `// SAFETY:` comment.

### Risk 3: gauss-quad API — RESOLVED

The `gauss-quad` crate may have a different API than assumed. Need to verify:
- Exact constructor: `GaussLegendre::new(order)?` or `GaussLegendre::init(order)`
- How nodes and weights are accessed: `.nodes()`, `.weights()`, or `.nodes_and_weights()`
- Whether it returns nodes on [-1,1] (standard) or a custom interval

**Resolution (confirmed):** Use `gauss-quad = "0.3"` (current version 0.3.1). API:
- Constructor: `GaussLegendre::new(degree)` where `degree: NonZeroUsize`. Returns `Self` directly (not `Result`).
- Node/weight access: `.nodes()` iterator, `.weights()` iterator, `.iter()` for `(node, weight)` pairs, `.as_node_weight_pairs()` for a slice
- Nodes are on [-1, 1] (standard Gauss-Legendre); `.integrate(a, b, f)` handles interval mapping internally
- Since `.integrate()` returns `f64` only, complex-valued kernel integration requires manual quadrature: iterate over `as_node_weight_pairs()`, apply the affine map from [-1,1] to [a,b], and sum `c64` contributions directly

### Risk 4: Near-neighbor bent junction approximation — RESOLVED (accepted trade-off)

Option A (use collinear formula for all junctions) introduces systematic error that grows with bend angle. For simple geometries (dipoles, Yagis), error is negligible. For fractal/meander-loaded antennas, error may be significant.

**Resolution:** This is an accepted design decision (documented in OPEN ITEM 4.2-A resolution, commit 6111147). No action needed for initial implementation. Option B (bend-angle correction via junction map) is documented as future work. The upgrade path is a formula substitution, not a structural change.

---

## Verification

After all steps complete:
- `cargo test --workspace` — all Rust tests pass
- `cargo clippy --workspace -- -D warnings` — no warnings
- `cargo fmt --check` — formatting clean
- `maturin develop && pytest tests/ -v` — Python integration tests pass
