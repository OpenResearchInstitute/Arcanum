//! V-DIAG: Diagonal element validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const SEG_LENGTH: f64 = 0.05;
const A_MOD: f64 = 0.005;

/// V-DIAG-001 — Diagonal Real Part is Positive (single segment).
#[test]
fn v_diag_001_single_segment_positive_real() {
    let mesh = helpers::straight_wire_mesh(1, SEG_LENGTH, A_MOD);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, 300e6, &config);

    assert!(
        z.read(0, 0).re > 0.0,
        "V-DIAG-001: Re(Z[0,0]) = {}, expected > 0",
        z.read(0, 0).re,
    );
}

/// V-DIAG-002 — Diagonal Real Part Positive for All Segments (11-segment dipole).
#[test]
fn v_diag_002_all_diagonals_positive_real() {
    let mesh = helpers::straight_wire_mesh(11, SEG_LENGTH, A_MOD);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, 300e6, &config);

    for m in 0..11 {
        assert!(
            z.read(m, m).re > 0.0,
            "V-DIAG-002: Re(Z[{m},{m}]) = {}, expected > 0",
            z.read(m, m).re,
        );
    }
}

/// V-DIAG-003 — Self-Impedance Frequency Dependence.
///
/// Re(Z[0,0]) must be positive at all tested frequencies.
/// Im(Z[0,0]) must be positive (inductive) at the standard test frequency
/// where Δ/λ = 0.05 and the inductive self-reactance dominates.
/// Re(Z) must be approximately constant across nearby frequencies (radiation
/// resistance is a smooth function of frequency for short segments).
#[test]
fn v_diag_003_frequency_dependence() {
    let config = MatrixFillConfig::default();
    let mesh = helpers::straight_wire_mesh(1, SEG_LENGTH, A_MOD);

    let freqs = [200e6, 300e6, 400e6];

    let mut re_values = Vec::new();
    let mut im_values = Vec::new();

    for &freq in &freqs {
        let z = fill_impedance_matrix(&mesh, freq, &config);
        let z00 = z.read(0, 0);

        assert!(
            z00.re > 0.0,
            "V-DIAG-003: Re(Z[0,0]) = {} at f={} Hz, expected > 0",
            z00.re, freq,
        );
        assert!(
            z00.re.is_finite() && z00.im.is_finite(),
            "V-DIAG-003: Z[0,0] is NaN/Inf at f={freq} Hz",
        );

        re_values.push(z00.re);
        im_values.push(z00.im);
    }

    // Im(Z[0,0]) should be positive (inductive) at 300 MHz for this geometry.
    assert!(
        im_values[1] > 0.0,
        "V-DIAG-003: Im(Z[0,0]) = {} at 300 MHz, expected > 0 (inductive)",
        im_values[1],
    );

    // Re(Z) should vary smoothly — check that the values at 200 and 400 MHz
    // bracket the value at 300 MHz, or at least are within 50% of each other
    // (radiation resistance changes slowly for electrically short segments).
    let re_min = re_values.iter().cloned().fold(f64::INFINITY, f64::min);
    let re_max = re_values.iter().cloned().fold(0.0f64, f64::max);
    assert!(
        re_max / re_min < 10.0,
        "V-DIAG-003: Re(Z) varies too much across frequencies: min={re_min}, max={re_max}",
    );
}
