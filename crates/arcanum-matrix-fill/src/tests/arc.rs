//! V-ARC: Arc segment matrix validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};
use std::f64::consts::PI;

const FREQ: f64 = 300e6;
const A_MOD: f64 = 0.005;

/// V-ARC-001 — Symmetry for Arc Segment Pair.
///
/// Z[0,1] = Z[1,0] for two separated arc segments.
#[test]
fn v_arc_001_symmetry() {
    let arc_radius = 0.1;
    let subtend_angle = PI / 6.0; // 30°
    let separation = 0.5;

    let mesh = helpers::two_arc_segments_separated(arc_radius, subtend_angle, A_MOD, separation);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let diff = (z.read(0, 1) - z.read(1, 0)).abs();
    assert!(
        diff < 1e-15,
        "V-ARC-001: |Z[0,1] - Z[1,0]| = {diff}, expected < ε_machine"
    );
}

/// V-ARC-002 — Arc Self-Impedance Real Part Positive.
#[test]
fn v_arc_002_self_impedance_positive_real() {
    let arc_radius = 0.1;
    let subtend_angle = PI / 6.0; // 30°

    let mesh = helpers::single_arc_segment(arc_radius, 0.0, subtend_angle, A_MOD);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    assert!(
        z.read(0, 0).re > 0.0,
        "V-ARC-002: Re(Z[0,0]) = {}, expected > 0",
        z.read(0, 0).re,
    );
}
