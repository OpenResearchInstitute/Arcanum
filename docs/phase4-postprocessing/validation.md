# Phase 4 — Post-Processing Validation

**Project:** Arcanum  
**Document:** `docs/phase4-postprocessing/validation.md`  
**Status:** DRAFT  
**Revision:** 0.1

---

## 1. Purpose

This document defines the validation cases for Phase 4, post-processing. Phase 4 computes all observable antenna parameters from the current vector [I]: radiation patterns, gain, directivity, radiated power, efficiency, and near fields.

Phase 4 validation is the final layer in the pipeline certification. It has the richest analytic ground truth of any phase — classical antenna theory provides exact pattern shapes, directivity values, and energy conservation identities that are geometry-independent.

**Scope boundary:** Phase 4 validation requires a correct [I] from Phase 3. A wrong current vector produces wrong patterns regardless of Phase 4 correctness. Phase 3 must be passing before Phase 4 results are interpreted. Where a Phase 4 case fails, the diagnostic question is always: is the pattern integration wrong, or is [I] wrong?

**Test case categories:**
- `V-PAT-XXX` — radiation pattern shape and symmetry
- `V-DIR-XXX` — directivity and gain values
- `V-POW-XXX` — radiated power and energy conservation
- `V-EFF-XXX` — radiation efficiency
- `V-NEAR-XXX` — near-field computation
- `V-RCS-XXX` — radar cross section (deferred, see Section 9)

---

## 2. Radiation Pattern Cases (V-PAT)

### V-PAT-001 — Half-Wave Dipole E-Plane Pattern Shape

**Setup:** 11-segment half-wave dipole along the z-axis, center-fed, 300 MHz. Far-field pattern computed in the E-plane (φ = 0, θ swept 0° to 180°).

**Expected pattern shape:**

The normalized far-field pattern of a half-wave dipole is:

```
F(θ) = cos(π/2 × cos(θ)) / sin(θ)
```

This produces a figure-8 pattern in the E-plane with:
- Maximum at θ = 90° (broadside, perpendicular to dipole axis)
- Nulls at θ = 0° and θ = 180° (along the dipole axis)
- No sidelobes

**Pass criterion:** The computed normalized pattern must match F(θ) to within ±0.5 dB at all θ angles where F(θ) > -20 dBi. Pattern nulls (θ = 0°, 180°) must be below -30 dBi.

---

### V-PAT-002 — Half-Wave Dipole H-Plane Pattern Shape

**Setup:** Same dipole as V-PAT-001. Pattern computed in the H-plane (θ = 90°, φ swept 0° to 360°).

**Expected pattern shape:** Omnidirectional — constant gain at all φ angles. The H-plane pattern of a z-axis dipole is independent of φ by symmetry.

**Pass criterion:**
```
max_φ |G(θ=90°, φ) - G_mean| < 0.1 dB
```

The pattern must be omnidirectional to within 0.1 dB. Any azimuthal variation is a numerical artifact.

---

### V-PAT-003 — Short Dipole Pattern Shape

**Setup:** 11-segment dipole, length = λ/10 = 0.1 m, center-fed, 300 MHz.

**Expected pattern shape:** The electrically short dipole pattern is:

```
F(θ) = sin(θ)
```

A pure sin(θ) donut pattern — the classic Hertzian dipole result. Maximum at θ = 90°, nulls at θ = 0° and 180°, no sidelobes. The pattern is rotationally symmetric about the dipole axis.

**Pass criterion:** Normalized pattern matches sin(θ) to within ±1 dB for all θ.

**Note:** The short dipole and the half-wave dipole have similar pattern shapes but the short dipole's pattern is exactly sin(θ) while the half-wave dipole's pattern has the cos(π/2 cosθ)/sinθ form. These are distinct — a test that accepts sin(θ) for a half-wave dipole or vice versa is wrong.

---

### V-PAT-004 — Pattern Symmetry for Symmetric Geometry

**Setup:** 11-segment half-wave dipole along the z-axis. Pattern computed at all (θ, φ) points on a 10° grid.

**Expected symmetry:**
- Azimuthal symmetry: G(θ, φ) = G(θ, φ + Δφ) for all Δφ (z-axis dipole is azimuthally symmetric)
- Upper/lower hemisphere symmetry: G(θ, φ) = G(π - θ, φ) for a center-fed symmetric dipole

**Pass criterion:**
```
max |G(θ, φ) - G(θ, φ+90°)| < 0.01 dB     (azimuthal symmetry)
max |G(θ, φ) - G(π-θ, φ)| < 0.01 dB        (upper/lower symmetry)
```

Any asymmetry in a geometrically symmetric structure is a numerical artifact indicating either a bug in the pattern integration or a non-symmetric current distribution from Phase 3.

---

### V-PAT-005 — 3-Element Yagi Pattern Has Single Main Beam

**Setup:** 3-element Yagi-Uda antenna (driven element, reflector, one director), driven element at 300 MHz. Far-field pattern on a 5° grid.

**Expected behavior:**
- Single main beam in the forward direction (toward director)
- Front-to-back ratio > 10 dB
- No grating lobes

**Pass criterion:** Exactly one beam maximum. Front-to-back ratio > 10 dB. This is a qualitative correctness check. It does not specify the exact gain value, only that the pattern has the expected topology.

---

## 3. Directivity and Gain Cases (V-DIR)

### V-DIR-001 — Isotropic Radiator Directivity

**Setup:** A single short segment carrying a uniform current. Approximating an isotropic radiator when the segment is electrically very short (Δ << λ). Length Δ = λ/1000, 300 MHz.

**Expected directivity:** An isotropic radiator has D = 1.0 (0 dBi) at all angles by definition.

**Note:** A true isotropic radiator cannot be physically realized — a short dipole has D = 1.5 (1.76 dBi). This case instead verifies the directivity normalization: the integral of the normalized pattern over all solid angles must equal 4π steradians.

**Pass criterion:**
```
∫∫ D(θ,φ) sin(θ) dθ dφ = 4π   (within 1%)
```

This is the fundamental normalization check on the directivity computation. It must pass for any antenna, not just a short dipole. If this integral does not equal 4π, the directivity is not correctly normalized. Where the rubber meets the road. If it had tires. 

---

### V-DIR-002 — Half-Wave Dipole Directivity

**Setup:** 11-segment half-wave dipole as in V-PAT-001.

**Expected maximum directivity:**
```
D_max = 1.64   (2.15 dBi)
```

This is the classical result for a half-wave dipole in free space. We better be able to stick the landing. 

**Pass criterion:** Computed D_max within ±0.2 dBi of 2.15 dBi.

**Convergence requirement:** D_max must converge toward 2.15 dBi as N increases from 11 to 41 segments.

---

### V-DIR-003 — Short Dipole Directivity

**Setup:** 11-segment dipole, length = λ/10, 300 MHz.

**Expected maximum directivity:**
```
D_max = 1.5   (1.76 dBi)
```

The Hertzian dipole (electrically short) has exactly D = 1.5. This is independent of dipole length as long as the antenna is electrically short.

**Pass criterion:** Computed D_max within ±0.3 dBi of 1.76 dBi.

---

### V-DIR-004 — Gain Equals Directivity for Lossless Antenna

**Setup:** 11-segment half-wave dipole, PEC wire (no ohmic loss), 300 MHz.

**Expected:**
```
G_max = D_max   (for η_rad = 100%)
```

Gain = Directivity when radiation efficiency η_rad = 1 (lossless antenna). For a PEC wire model, there are no ohmic losses and all input power is radiated.

**Pass criterion:**
```
|G_max - D_max| / D_max < 0.01   (1% tolerance)
```

---

### V-DIR-005 — Gain Reduced by Ohmic Loss

**Setup:** 11-segment half-wave dipole, copper wire (σ = 5.8×10⁷ S/m), 300 MHz, wire radius 0.001 m.

**Expected:**
```
G_max < D_max
η_rad = G_max / D_max < 1.0
```

The gain must be less than the directivity when ohmic losses are present. The efficiency reduction is small for copper at 300 MHz (copper loss is negligible for typical wire antennas at HF/VHF) but must be non-zero and in the physically correct direction.

**Pass criterion:** G_max < D_max. η_rad is in the range (0.99, 1.0) for copper at 300 MHz.

---

## 4. Power and Energy Conservation Cases (V-POW)

### V-POW-001 — Energy Conservation: Radiated Power Equals Input Power (Lossless)

**Setup:** 11-segment half-wave dipole, PEC wire, 300 MHz, 1V source.

**Expected:**
```
P_rad = P_in = (1/2) Re(V_s × I*[m_src])
```

For a lossless antenna, all input power is radiated. The radiated power computed by integrating the far-field pattern over all angles must equal the input power computed from the source voltage and current.

**Pass criterion:**
```
|P_rad - P_in| / P_in < 0.01   (1% tolerance)
```

This is a fundamental energy conservation check. A failure indicates either wrong pattern integration in Phase 4 or wrong current computation in Phase 3.

---

### V-POW-002 — Radiated Power Increases with Source Voltage Squared

**Setup:** Half-wave dipole, PEC, 300 MHz. Solve at V_s = {0.5V, 1.0V, 2.0V}.

**Expected:**
```
P_rad(2V) = 4 × P_rad(1V)
P_rad(0.5V) = 0.25 × P_rad(1V)
```

The system is linear — doubling the source voltage doubles all currents and quadruples all power quantities.

**Pass criterion:** P_rad scales as V_s² to within 0.1%.

---

### V-POW-003 — Power Balance with Ohmic Loss

**Setup:** Half-wave dipole, copper wire, 300 MHz.

**Expected:**
```
P_in = P_rad + P_loss
P_loss = Σ_m (1/2) |I[m]|² × R_seg(m)
```

where R_seg(m) is the ohmic resistance of segment m from Phase 3.

**Pass criterion:**
```
|P_in - P_rad - P_loss| / P_in < 0.01
```

The power balance must close: input power equals radiated power plus ohmic loss.

---

## 5. Near-Field Cases (V-NEAR)

### V-NEAR-001 — Near-Field On-Axis for Short Dipole

**Setup:** Single short segment (Δ = λ/100), carrying 1A current, 300 MHz. Near electric field computed at points along the z-axis (dipole axis) at distances r = {0.1λ, 0.5λ, 1.0λ, 2.0λ}.

**Expected behavior:**
- Near field (r << λ): E_z ∝ 1/r³ (quasi-static dominant term)
- Far field (r >> λ): E_z ∝ 1/r (radiation dominant term)
- Transition at r ≈ λ/(2π)

**Pass criterion:** The near-to-far field transition is present. E_z × r is not constant at r = 0.1λ (near field dominates) but is approximately constant at r = 2.0λ (far field dominates).

---

### V-NEAR-002 — Near-Field Symmetry

**Setup:** 11-segment half-wave dipole along z-axis, 300 MHz. Near electric field computed on a grid in the xz-plane and the yz-plane.

**Expected:** E-field in xz-plane equals E-field in yz-plane at corresponding points (azimuthal symmetry of z-axis dipole).

**Pass criterion:**
```
|E_xz(r,θ) - E_yz(r,θ)| / |E_xz(r,θ)| < 0.01
```

at all grid points where |E| > 1% of maximum.

---

### V-NEAR-003 — Near-Field Continuity

**Setup:** Near electric field computed on a grid of 20×20 points in a plane near a half-wave dipole.

**Expected:** The field varies smoothly. No discontinuities or NaN values anywhere on the grid, including points close to the antenna structure (but not inside the wire, which is physically excluded).

**Pass criterion:** All 400 field values are finite and non-NaN. The field magnitude varies smoothly with no jumps greater than a factor of 10 between adjacent grid points (unless the grid passes very close to the wire surface where large but finite fields are expected).

---

## 6. Radiation Efficiency Cases (V-EFF)

### V-EFF-001 — PEC Wire Efficiency Is Unity

**Setup:** Half-wave dipole, PEC (no conductivity load), 300 MHz.

**Expected:**
```
η_rad = P_rad / P_in = 1.0
```

**Pass criterion:** η_rad > 0.999.

---

### V-EFF-002 — Efficiency Decreases with Added Resistance

**Setup:** Half-wave dipole, PEC, 300 MHz. Two solves:
- Run A: no load (baseline)
- Run B: 73 Ω resistive load at center (equal to radiation resistance)

**Expected:**
```
η_rad(Run B) ≈ 0.5   (load dissipates half the input power)
```

Adding a series resistance equal to the radiation resistance halves the efficiency.

**Pass criterion:** η_rad(Run B) in range (0.45, 0.55).

---

## 7. Helix Antenna Cases (V-HEL)

These cases are specifically motivated by ORI's use case — ground station helix antennas for satellite communication.

### V-HEL-001 — Axial-Mode Helix Has End-Fire Pattern

**Setup:** 5-turn axial-mode helix, circumference ≈ 1λ, pitch ≈ λ/4, 300 MHz, over PEC ground plane. Far-field pattern on 5° grid.

**Expected:**
- End-fire pattern: main beam along the helix axis (θ = 0°, pointing away from ground plane)
- Half-power beamwidth (HPBW) ≈ 52° / √(N_turns × C/λ) ≈ 52° / √5 ≈ 23° (Kraus formula)
- Gain ≈ 10-12 dBi (Kraus estimate for 5 turns)

**Pass criterion:**
- Main beam at θ = 0° ± 10°
- Gain in range 8–14 dBi (broad tolerance for Kraus approximation)
- HPBW in range 15°–35°

**Rationale:** This is Arcanum's primary ORI use case. A helix that produces the wrong pattern topology (broadside instead of end-fire, or multiple competing beams) indicates a fundamental failure in either the helix geometry (Phase 1) or the current computation (Phases 2-3). The Kraus formulas are approximate. Wide tolerances are appropriate. Special thanks to Kerry Banke N6IZW for the inspiration and illumination for this and other tests in this phase.

---

### V-HEL-002 — CMoM vs Polygon Approximation Convergence

**Setup:** 5-turn axial-mode helix, 8 segments per turn (40 total), 300 MHz.

**Test:** Compute gain and input impedance twice:
- Run A: using CMoM curved arc segments (Phase 1 native helix discretization)
- Run B: using straight-segment approximation of the same helix (each arc segment replaced by its chord)

**Expected:** Run A (CMoM) produces a more accurate result than Run B (straight-segment approximation). Specifically, Run A impedance should be closer to the converged value obtained with 32 segments per turn.

**Pass criterion:** At 8 segments per turn, the CMoM result must be closer to the high-N reference than the straight-segment result. The difference in gain between the two methods must be measurable (> 0.1 dB). If they agree perfectly at 8 segments per turn, CMoM is not providing its advertised advantage.

**Rationale:** This case directly validates Arcanum's core technical claim. If CMoM curved segments do not outperform straight-segment approximation for helix antenna modeling, the project's primary motivation is not yet demonstrated. We go back, find the shortfall, fix it, test again. 

---

## 8. Validation Procedure

Far-field pattern cases (V-PAT, V-DIR) require computing the pattern on a (θ, φ) grid. The minimum grid resolution for all cases is 5° in both θ and φ (37 × 73 = 2701 points). Coarser grids may miss narrow features. Solid test.

All power cases (V-POW, V-EFF) require integrating the far-field pattern over all solid angles. The numerical integration uses the same (θ, φ) grid with appropriate sin(θ) weighting. 

Each case is implemented as a Rust integration test running the full Phase 1 → Phase 2 → Phase 3 → Phase 4 pipeline.

Required figures committed to `docs/phase4-postprocessing/figures/`:
- E-plane and H-plane patterns for V-PAT-001 and V-PAT-002
- 3D pattern visualization for V-HEL-001
- CMoM vs straight-segment convergence plot for V-HEL-002

---

## 9. Deferred Cases

**Radar Cross Section (RCS):** Phase 4 is specified to compute RCS but validation cases are deferred until the far-field and near-field cases are passing. RCS requires incident plane wave excitation (EXTYPE = 1) which is not in initial scope. Interested? Get in touch. 

---

## 10. References

- `docs/phase4-postprocessing/math.md` — far-field integral; directivity formula; near-field Green's function
- `docs/phase4-postprocessing/design.md` — pattern computation architecture
- `docs/phase3-matrix-solve/validation.md` — Phase 3 must pass before Phase 4 results are meaningful
- Balanis, C.A. — *Antenna Theory* — half-wave dipole pattern; directivity formulas; energy conservation
- Kraus, J.D. — *Antennas* — axial-mode helix gain and beamwidth formulas
- `docs/nec-import/reference-decks/helix-over-ground.nec` — ORI primary use case reference deck

---

*Arcanum — Open Research Institute*
