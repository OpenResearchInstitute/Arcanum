// tests/mod.rs
//
// Rust unit tests for arcanum-nec-import.
//
// Each submodule corresponds to a section in docs/nec-import/validation.md:
//   parse    — V-PARSE  field parsing
//   fmt      — V-FMT    input format tolerance
//   errors   — V-ERR    hard error cases
//   warnings — V-WARN   non-fatal warning cases
//   real     — V-REAL   real-world reference decks

mod errors;
mod fmt;
mod parse;
mod real;
mod warnings;

// ─────────────────────────────────────────────────────────────────────────────
// Shared helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Assert that two f64 values are within 1e-9 of each other.
macro_rules! approx_eq {
    ($a:expr, $b:expr) => {{
        let a = $a as f64;
        let b = $b as f64;
        assert!(
            (a - b).abs() < 1e-9,
            "approx_eq failed: left={} right={} diff={}",
            a,
            b,
            (a - b).abs()
        );
    }};
    ($a:expr, $b:expr, $eps:expr) => {{
        let a = $a as f64;
        let b = $b as f64;
        assert!(
            (a - b).abs() < $eps as f64,
            "approx_eq failed: left={} right={} diff={} eps={}",
            a,
            b,
            (a - b).abs(),
            $eps
        );
    }};
}

pub(super) use approx_eq;
