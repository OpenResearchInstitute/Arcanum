# Phase 1 — Convergence Plot Requirement

**Project:** Arcanum  
**Document:** `docs/phase1-geometry/plots.md`  
**Status:** CLOSED — Option 3 approved 2026-04-17  
**Revision:** 0.2

---

## 1. The Requirement

`validation.md` Section 10 contains the following sentence:

> *"Convergence plots, demonstrating that geometric continuity and endpoint
> accuracy hold as N increases, are required for V-HEL-002 and V-ARC-001
> before Phase 1 implementation is marked complete."*

This was written as a quality gate: before Phase 1 could be called done, someone
had to produce plots showing that the discretization behaves correctly as the
number of segments N increases.

---

## 2. What Was Actually Done

The existing tests address both targeted cases:

**V-HEL-002** (`tests/helix.rs::v_hel_002_five_turn_endpoint_continuity`):
- Runs a 5-turn helix at N = 40 segments
- Asserts that the gap between every adjacent segment endpoint pair is less than
  1×10⁻¹² m (double-precision rounding only)
- Asserts that the final z-coordinate equals the expected total axial length

**V-ARC-001** (`tests/arc.rs::v_arc_001_semicircle_4_segments`):
- Runs a semicircular arc at N = 4 segments
- Asserts endpoint coordinates to 4 decimal places (0.0001 m tolerance)
- Asserts that y = 0 for all endpoints (arc remains in XZ plane)

These are point-in-time checks at fixed N values. No sweep over varying N has
been written.

---

## 3. Why the Requirement May Be Unnecessary

Both helix and arc discretizations use **closed-form parametric evaluation**.
Endpoint coordinates are computed directly from the parametric expressions at
`t = k/N` and `t = (k+1)/N` — there is no incremental accumulation of error.

For the helix:
```
r(t) = (A(t) cos(2π N_turns t),  A(t) sin(2π N_turns t),  HL · t)
```

For the arc:
```
r(θ) = (R cos θ,  0,  R sin θ)     θ_k = θ₁ + k/N · (θ₂ - θ₁)
```

Because each endpoint is evaluated independently at its exact parameter value,
geometric continuity (end of segment k = start of segment k+1) holds to
double-precision rounding for any N ≥ 1. This is a mathematical property of
the parametric form, not an empirical observation. A convergence plot would
confirm what is already proven by construction.

The convergence plots requirement is more appropriate for situations where the
discretization might accumulate error — for example, if endpoints were computed
by stepping from a previous endpoint rather than evaluating the curve directly.
The implementation was explicitly designed to avoid that (see the comments in
`discretize.rs`).

---

## 4. Options

**Option 1 — Implement the plots**

Write a small program (Rust or Python) that:
- Sweeps N from 1 to some large value (e.g. 1, 2, 4, 8, 16, 32, 64, 128)
- For each N, computes the maximum endpoint gap and maximum coordinate error
- Prints a table or produces a matplotlib plot

This defers closing Phase 1 and adds a deliverable that confirms what the math
already says. The value is documentation and trust, not new information.

**Option 2 — Defer to Phase 2**

Convergence of the *geometric* discretization is meaningful only in the context
of *electromagnetic* convergence — does the MoM solution converge as N
increases? That question belongs to Phase 2, which has access to both the
geometry and the computed currents. A geometry-only convergence check answers
a less interesting question.

Under this option, the sentence in `validation.md` Section 10 is replaced with
a forward reference to Phase 2 validation.

**Option 3 — Remove the requirement**

The algebraic tests already demonstrate the key property at the tolerance
required by the validation spec. The closed-form parametric implementation
makes geometric continuity a mathematical certainty, not something that needs
empirical verification across multiple N values. Remove the sentence from
`validation.md` Section 10 and close Phase 1.

---

## 5. Recommendation

Option 3 is recommended. The requirement was written defensively, anticipating
an incremental implementation that might accumulate error. The implementation
uses closed-form evaluation instead, making the requirement moot. Removing it
is the honest reflection of what the tests cover and why.

If convergence plots are wanted, they belong in the Phase 2 validation
document, where they can show solution convergence (currents, impedance, gain)
as a function of N — a much more useful result.

---

## 6. Decision Required

Choose one:

- [ ] Option 1 — implement convergence plots before closing Phase 1
- [ ] Option 2 — defer to Phase 2 validation
- [x] **Option 3 — remove the requirement; algebraic tests are sufficient** ✓ approved 2026-04-17

---

*Arcanum — Open Research Institute*
