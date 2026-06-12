//! V-NEAR: Near-singular element validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const FREQ: f64 = 300e6;
const SEG_LENGTH: f64 = 0.05;

/// V-NEAR-001 — Self-Impedance Does Not Diverge.
///
/// Re(Z[0,0]) remains finite and positive for all tested radii.
#[test]
fn v_near_001_self_impedance_finite() {
    let config = MatrixFillConfig::default();
    let radii = [SEG_LENGTH / 10.0, SEG_LENGTH / 100.0, SEG_LENGTH / 1000.0];

    for &a in &radii {
        let mesh = helpers::straight_wire_mesh(1, SEG_LENGTH, a);
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        let z_self = z.read(0, 0);

        assert!(
            z_self.re.is_finite() && z_self.im.is_finite(),
            "V-NEAR-001: Z[0,0] contains NaN/Inf at a={a}: {:?}",
            z_self,
        );
        assert!(
            z_self.re > 0.0,
            "V-NEAR-001: Re(Z[0,0]) = {} at a={a}, expected > 0",
            z_self.re,
        );
    }
}

/// V-NEAR-002 — Near-Neighbor Element Does Not Diverge.
#[test]
fn v_near_002_near_neighbor_finite() {
    let a_thin = 5e-5; // Δ/1000
    let mesh = helpers::straight_wire_mesh(2, SEG_LENGTH, a_thin);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);
    let z_01 = z.read(0, 1);

    assert!(
        z_01.re.is_finite() && z_01.im.is_finite(),
        "V-NEAR-002: Z[0,1] contains NaN/Inf: {:?}",
        z_01,
    );
}

/// V-NEAR-003 — Self vs Near-Neighbor Magnitude Ordering.
///
/// For a uniform dipole, |Z[5,5]| > |Z[5,4]| > |Z[5,3]| > ... > |Z[5,0]|.
#[test]
fn v_near_003_magnitude_ordering() {
    let a_mod = 0.005;
    let mesh = helpers::straight_wire_mesh(11, SEG_LENGTH, a_mod);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let center = 5;
    let magnitudes: Vec<f64> = (0..=5)
        .map(|offset| z.read(center, center - offset).abs())
        .collect();

    // magnitudes[0] = |Z[5,5]|, magnitudes[1] = |Z[5,4]|, ..., magnitudes[5] = |Z[5,0]|
    for i in 1..magnitudes.len() {
        assert!(
            magnitudes[i - 1] > magnitudes[i],
            "V-NEAR-003: |Z[5,{}]| = {} should be > |Z[5,{}]| = {}",
            center - (i - 1),
            magnitudes[i - 1],
            center - i,
            magnitudes[i],
        );
    }
}
