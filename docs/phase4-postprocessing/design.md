# Phase 4 — Post-Processing Design

**Project:** Arcanum  
**Document:** `docs/phase4-postprocessing/design.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document describes the architecture of Phase 4 post-processing. Phase 4 computes all observable antenna parameters from the solved current vector [I]: far-field radiation patterns, directivity, gain, radiated power, radiation efficiency, and near fields.

Mathematical derivations are in `math.md`. Validation cases are in `validation.md`. This document is concerned with how the math is implemented.

---

## 2. Interface

### 2.1 Entry Point

```rust
pub fn postprocess(
    solve_result: &SolveResult,
    mesh: &Mesh,
    frequency: f64,
    requests: &OutputRequests,
    config: &PostprocessConfig,
) -> PostprocessResult
```

- `solve_result` — from Phase 3. Contains the current vector [I], input impedances, and the factorization. Read-only.
- `mesh` — from Phase 1. Provides segment geometry (parametric forms, arc lengths, radii). Read-only.
- `frequency` — in Hz. No MHz conversion in Phase 4.
- `requests` — from the NEC parser (RP, NE, NH cards). Specifies what to compute and where.
- `config` — quadrature and accuracy configuration.
- Returns `PostprocessResult` containing all computed quantities and context notes.

### 2.2 Phase Boundary

Phase 4 is the terminal phase. It receives from Phase 3 and produces final results for the user. It does not call any upstream phase at runtime.

```
Phase 3 → SolveResult (currents, input impedances)
Phase 1 → Mesh (read-only)
NEC parser → OutputRequests (RP, NE, NH)
                    ↓
            Phase 4: postprocess()
                    ↓
            PostprocessResult → user / Python layer
```

---

## 3. Data Structures

### 3.1 PostprocessResult

```rust
pub struct PostprocessResult {
    pub pattern: Option<RadiationPattern>,     // far-field pattern (if RP requested)
    pub near_field_e: Option<NearFieldGrid>,   // near E-field (if NE requested)
    pub near_field_h: Option<NearFieldGrid>,   // near H-field (if NH requested)
    pub power: PowerBudget,                    // always computed
    pub context: PatternContext,               // notes for user display
}
```

### 3.2 RadiationPattern

```rust
pub struct RadiationPattern {
    pub theta: Vec<f64>,              // θ angles in radians
    pub phi: Vec<f64>,                // φ angles in radians
    pub directivity: Array2<f64>,     // D(θ,φ) — linear, not dB
    pub gain: Array2<f64>,            // G(θ,φ) — linear
    pub e_theta: Array2<Complex<f64>>, // E_θ far-field component
    pub e_phi: Array2<Complex<f64>>,  // E_φ far-field component
    pub axial_ratio: Array2<f64>,     // polarization axial ratio
    pub d_max: f64,                   // maximum directivity (linear)
    pub g_max: f64,                   // maximum gain (linear)
    pub theta_max: f64,               // θ of maximum gain (radians)
    pub phi_max: f64,                 // φ of maximum gain (radians)
}
```

All pattern values are stored in linear (not dB) scale internally. Conversion to dBi is performed at output time in the Python layer. This avoids repeated dB conversion and keeps the internal representation unambiguous. Similar approach to MHz vs Hz.

### 3.3 PowerBudget

```rust
pub struct PowerBudget {
    pub p_input: f64,        // input power from Phase 3 (W)
    pub p_radiated: f64,     // radiated power from far-field integration (W)
    pub p_loss: f64,         // ohmic loss = p_input - p_radiated (W)
    pub efficiency: f64,     // η_rad = p_radiated / p_input (dimensionless)
    pub p_rad_check: f64,    // radiated power from impedance matrix method (W)
    pub power_balance_error: f64, // |p_radiated - p_rad_check| / p_input
}
```

`p_radiated` is computed by far-field integration. `p_rad_check` is computed from the impedance matrix method (Section 9 of `math.md`). `power_balance_error` is the cross-check. It should be < 1% for a well-conditioned simulation. A value above 5% generates a warning in `PatternContext`. Should is doing a lot of work here, but if everything has been validated all along the way, we expect to see excellent results. 

### 3.4 PatternContext

```rust
pub struct PatternContext {
    pub notes: Vec<ContextNote>,
    pub warnings: Vec<ContextWarning>,
}

pub struct ContextNote {
    pub code: NoteCode,
    pub message: String,    // human-readable; complete sentence explaining the situation
}

pub enum NoteCode {
    GroundPlaneUpperHemisphereOnly,
    AdaptiveQuadratureUsed,
    NearFieldCloseToWire,
    PatternNullHandled,
}

pub enum ContextWarning {
    PowerBalanceError { error_fraction: f64 },
    HighKDeltaQuadrature { segment_index: usize, k_delta: f64 },
    ObservationPointNearWire { distance_radii: f64 },
}
```

Context notes are informational. They explain what was done and why. Context warnings indicate a potential accuracy concern. Neither is an error. The Python layer displays notes and warnings alongside results.

**Ground plane note (issued automatically when GroundType::PEC):**
```
"Ground plane present. Pattern restricted to upper hemisphere (θ ∈ [0°, 90°]).
Image segments in the mesh contribute to the upper hemisphere pattern through
constructive and destructive interference with the real antenna. The lower
hemisphere (θ > 90°) is not physical and is not computed."
```

This note is always issued when a PEC ground plane is present, regardless of whether the user requested it. It is not a warning. It explains expected behavior.

---

## 4. Far-Field Pattern Computation Pipeline

### Step 1 — Build Observation Grid

From the RP card parameters (THETS, PHIS, DTHS, DPHS, NTHETA, NPHI), build the (θ,φ) observation grid:

```rust
let theta: Vec<f64> = (0..n_theta)
    .map(|i| (thets + i as f64 * dths).to_radians())
    .collect();
let phi: Vec<f64> = (0..n_phi)
    .map(|j| (phis + j as f64 * dphs).to_radians())
    .collect();
```

If a ground plane is present, restrict θ to [0°, 90°] and issue the ground plane note. Any RP card requesting θ > 90° with a ground plane present is silently clamped — do not error, do clamp, do issue the note.

### Step 2 — Precompute Segment Quadrature Points

For each segment m, precompute the Gauss-Legendre quadrature points and weights, and evaluate the parametric quantities at each point:

```rust
struct SegmentQuadrature {
    positions: Vec<Vec3>,      // r_m(τ_i) at each quadrature point
    tangents: Vec<Vec3>,       // t̂_m(τ_i) at each quadrature point
    arc_weights: Vec<f64>,     // w_i × |r'_m(τ_i)| — combined GL weight and arc length element
    quadrature_order: usize,   // p used for this segment
}
```

The quadrature order for segment m is:

```rust
let k_delta = k * segment_arc_length(m, mesh);
let p = if k_delta < 1.0 {
    8
} else {
    (2.0 * k_delta / PI).ceil() as usize + 4  // extra margin beyond minimum
}.max(8).min(64);
```

**Note:** The adaptive order trigger is `k_delta = k × Δ_arc` — electrical arc length, not physical length or frequency. A λ/20 segment at any frequency has k_delta = π/10 and requires only p = 8. A λ/2 segment at any frequency has k_delta = π and requires higher order. Frequency is irrelevant (this idea can take some getting used to). Electrical length is the correct criterion. See `math.md` Section 5.2.

Segment quadrature is precomputed once and reused across all observation directions. This is the dominant memory vs compute trade-off in Phase 4. Storing all quadrature points costs O(N×p) memory but avoids recomputing segment geometry for each of the O(N_θ × N_φ) observation directions.

### Step 3 — Compute Radiation Vector Per Direction

For each observation direction r̂(θ,φ), accumulate the radiation vector N:

```rust
// Parallelized over observation directions via Rayon
pattern_grid.par_iter_mut().for_each(|(theta_i, phi_j, n_vec)| {
    let r_hat = spherical_to_cartesian(theta_i, phi_j);
    *n_vec = Complex::zero_vec3();
    for m in 0..n_segments {
        let current = solve_result.currents[m];
        for q in 0..seg_quad[m].quadrature_order {
            let phase = k * r_hat.dot(seg_quad[m].positions[q]);
            let weight = seg_quad[m].arc_weights[q];
            *n_vec += current * weight * seg_quad[m].tangents[q]
                    * Complex::from_polar(1.0, phase);
        }
    }
});
```

This is O(N_θ × N_φ × N × p). This is the dominant computational cost of Phase 4. For typical cases (N_θ = 37, N_φ = 73, N = 100, p = 8): 37 × 73 × 100 × 8 ≈ 2.2 million operations. Fast!

### Step 4 — Project to θ and φ Components

For each direction, project N to spherical components:

```rust
let n_theta = n_vec.dot(theta_hat(theta_i, phi_j));
let n_phi = n_vec.dot(phi_hat(phi_j));
```

Handle θ = 0° and θ = 180° explicitly:
```rust
if theta_i.abs() < 1e-10 || (PI - theta_i).abs() < 1e-10 {
    // Pattern null on dipole axis — return zero, not NaN
    e_theta = Complex::zero();
    e_phi = Complex::zero();
} else {
    e_theta = prefactor * n_theta;
    e_phi = prefactor * n_phi;
}
```

This guard prevents NaN from the cos(π/2 cos(θ))/sin(θ) limit. See `math.md` Section 6.3. Zero is the physically correct value at these angles for any antenna with a null on axis.

### Step 5 — Compute Radiation Intensity and Integrate

From E_θ and E_φ, compute radiation intensity U(θ,φ). Integrate over the grid to get P_rad:

```rust
let p_rad = integrate_over_sphere(&u_grid, &theta, &phi);
```

Sphere integration uses the trapezoidal rule with sin(θ) weighting. For uniform grids this is sufficient. The trapezoidal rule is spectrally accurate for smooth periodic functions in φ. For non-uniform grids, Simpson's rule is used.

### Step 6 — Compute Directivity and Gain

```rust
let u_iso = p_rad / (4.0 * PI);
let directivity = u_grid.mapv(|u| u / u_iso);
let p_in = solve_result.power.p_input;
let gain = directivity.mapv(|d| d * p_rad / p_in);
```

---

## 5. Near-Field Computation Pipeline

### Step 1 — Build Observation Grid

From NE/NH card parameters, build the 3D Cartesian observation grid. Detect any observation point within 3 wire radii of any segment and issue a `NearFieldCloseToWire` context warning for each such point.

Observation points inside any wire (distance < wire radius) return `NaN` with a hard warning. These are physically meaningless and must not silently produce wrong values.

### Step 2 — Compute Fields at Each Point

For each observation point r, evaluate the near-field electric and magnetic fields using the full Green's function (no far-field approximation):

```rust
// Parallelized over observation points via Rayon
near_field_grid.par_iter_mut().for_each(|(point, e_field, h_field)| {
    *e_field = compute_near_e(point, &solve_result.currents, mesh, k);
    *h_field = compute_near_h(point, &solve_result.currents, mesh, k);
});
```

Quadrature order for near-field segments follows the same kΔ criterion as far-field, with additional elevation for near-wire observation points.

---

## 6. Power Budget Computation

### 6.1 Primary Path — Far-Field Integration

```rust
power_budget.p_radiated = integrate_over_sphere(&u_grid, &theta, &phi);
power_budget.p_input = solve_result.power.p_input;
power_budget.p_loss = p_input - p_radiated;
power_budget.efficiency = p_radiated / p_input;
```

### 6.2 Cross-Check Path — Impedance Matrix Method

```rust
// P_rad = (1/2) I^H Re(Z) I
let p_rad_check = 0.5 * currents.conj_transpose()
    .dot(&z_real.dot(&currents))
    .re;
power_budget.p_rad_check = p_rad_check;
power_budget.power_balance_error =
    (p_radiated - p_rad_check).abs() / p_input;
```

If `power_balance_error > 0.05` (5%), issue a `PowerBalanceError` context warning. This cross-check is the diagnostic that distinguishes a Phase 2 error (wrong [Z], wrong `p_rad_check`) from a Phase 4 integration error (wrong `p_radiated`). If both paths agree and the residual from Phase 3 is small, the simulation is internally consistent.

---

## 7. Configuration

```rust
pub struct PostprocessConfig {
    pub min_quadrature_order: usize,        // default 8
    pub max_quadrature_order: usize,        // default 64
    pub k_delta_threshold: f64,             // default 1.0 — triggers adaptive order
    pub near_wire_warning_radii: f64,       // default 3.0 — wire radii
    pub power_balance_warning_threshold: f64, // default 0.05 (5%)
    pub upper_hemisphere_only: Option<bool>, // None = auto (ground plane present)
}
```

`upper_hemisphere_only: None` means Phase 4 decides automatically based on ground plane presence. The user can override — `Some(true)` forces upper hemisphere regardless of ground plane; `Some(false)` forces full sphere even with a ground plane (useful for verification that image contributions are zero in lower hemisphere).

---

## 8. Axial Ratio and Polarization

For each far-field direction, the axial ratio (AR) characterizes the polarization ellipse:

```rust
let ar = axial_ratio(e_theta, e_phi);
```

where:

```
AR = (|E_θ| + |E_φ|) / (|E_θ| - |E_φ|)   (simplified for linear polarization check)
```

AR = 1 indicates circular polarization (equal E_θ and E_φ magnitudes). AR = ∞ indicates linear polarization (one component zero). The axial-mode helix produces near-circular polarization — AR near 1 over the main beam is an additional validation point for V-HEL-001.

---

## 9. What Phase 4 Does Not Do

Phase 4 does not:

- **Re-solve the linear system.** It receives [I] from Phase 3.
- **Modify the mesh.** Read-only reference to Phase 1.
- **Re-fill the impedance matrix.** The cross-check uses Re([Z]) from Phase 2 via `SolveResult`.
- **Compute RCS for incident plane waves.** Deferred — requires EXTYPE = 1 plane wave excitation not in initial scope.
- **Produce plots or visualizations.** Output formatting is the Python layer's responsibility. Phase 4 produces numerical arrays. The Python layer calls Matplotlib.

---

## 10. Open Items

1. **Sphere integration accuracy.** The trapezoidal rule on a uniform (θ,φ) grid is spectrally accurate for smooth functions but has a well-known inaccuracy at θ = 0° and θ = 180° where sin(θ) = 0. The integration weight approaches zero at the poles, so the error contribution is small. For high-accuracy directivity computation (better than 0.1 dBi), a Gauss-Legendre rule in θ with uniform spacing in φ should be considered. Track as a GitHub issue.

2. **Near-field curl computation.** The magnetic near field requires ∇ × A. The initial implementation uses finite differences on the observation grid. Analytic differentiation of the Green's function under the integral sign is more accurate but more complex. Defer analytic curl to a future revision.

3. **Front-to-back ratio.** A commonly requested antenna metric. The ratio of gain in the main beam direction to gain in the opposite direction. Not currently in `OutputRequests` or `RadiationPattern`. Add as a derived quantity computed from the pattern grid. Low effort, high user value. Take this job and make it happen!

4. **Half-power beamwidth (HPBW).** Similarly high user value. Requires finding the -3 dBi contour of the pattern. Add alongside front-to-back ratio.

---

## 11. Anticipated Repository Layout

```
src/
├── postprocess/
│   ├── mod.rs                  ← pub fn postprocess()
│   ├── far_field.rs            ← radiation vector; E/H field computation
│   ├── near_field.rs           ← near-field E and H
│   ├── power.rs                ← PowerBudget; far-field integration; cross-check
│   ├── pattern.rs              ← RadiationPattern; directivity; gain; axial ratio
│   ├── quadrature.rs           ← adaptive order selection; kΔ criterion
│   ├── context.rs              ← PatternContext; ContextNote; ContextWarning
│   ├── config.rs               ← PostprocessConfig
│   ├── results.rs              ← PostprocessResult; NearFieldGrid
│   └── tests/
│       ├── pattern_tests.rs    ← V-PAT cases
│       ├── dir_tests.rs        ← V-DIR cases
│       ├── power_tests.rs      ← V-POW cases
│       ├── eff_tests.rs        ← V-EFF cases
│       ├── near_tests.rs       ← V-NEAR cases
│       └── helix_tests.rs      ← V-HEL cases
```

---

## 12. References

- `docs/phase4-postprocessing/math.md` — radiation integral; directivity formula; near-field Green's function; Kraus helix formulas
- `docs/phase4-postprocessing/validation.md` — test cases
- `docs/phase3-matrix-solve/design.md` — SolveResult; PowerBudget.p_input
- `docs/phase2-matrix-fill/design.md` — ZMatrix; Re([Z]) for power cross-check
- Balanis, C.A. — *Antenna Theory* — far-field expressions; directivity
- Kraus, J.D. — *Antennas* — axial-mode helix; axial ratio

---

*Arcanum — Open Research Institute*
