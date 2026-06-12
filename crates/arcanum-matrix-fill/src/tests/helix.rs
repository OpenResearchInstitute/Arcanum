//! V-HEL: Helix segment matrix validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const FREQ: f64 = 300e6;
const A_MOD: f64 = 0.005;

/// V-HEL-001 — Symmetry for Helix Segment Pair.
///
/// Z[0,1] = Z[1,0] for first and last segments of an 8-segment helix.
#[test]
fn v_hel_001_symmetry() {
    let mesh = helpers::two_helix_segments(
        0.05,  // helix radius
        0.4,   // total axial length
        1.0,   // 1 turn
        A_MOD,
    );
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let diff = (z.read(0, 1) - z.read(1, 0)).abs();
    assert!(
        diff < 1e-15,
        "V-HEL-001: |Z[0,1] - Z[1,0]| = {diff}, expected < ε_machine"
    );
}

/// V-HEL-002 — Helix Self-Impedance Real Part Positive.
#[test]
fn v_hel_002_self_impedance_positive_real() {
    let mesh = helpers::single_helix_segment(
        0.05,  // helix radius
        0.4,   // total axial length
        1.0,   // 1 turn
        0,     // first segment
        8,     // of 8
        A_MOD,
    );
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    assert!(
        z.read(0, 0).re > 0.0,
        "V-HEL-002: Re(Z[0,0]) = {}, expected > 0",
        z.read(0, 0).re,
    );
}
