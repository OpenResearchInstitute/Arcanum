// Phase 1 test suite — one file per validation category

mod arc;
mod ground;
mod helix;
mod linear;
mod tagmap;
mod transforms;
mod warnings;

/// Approximate equality helper for f64, matching the nec-import convention.
macro_rules! approx_eq {
    ($a:expr, $b:expr) => {
        assert!(
            ($a - $b).abs() < 1e-9,
            "approx_eq failed: {} ≠ {} (diff = {})",
            $a,
            $b,
            ($a - $b).abs()
        )
    };
    ($a:expr, $b:expr, $tol:expr) => {
        assert!(
            ($a - $b).abs() < $tol,
            "approx_eq failed: {} ≠ {} (diff = {}, tol = {})",
            $a,
            $b,
            ($a - $b).abs(),
            $tol
        )
    };
}
pub(crate) use approx_eq;
