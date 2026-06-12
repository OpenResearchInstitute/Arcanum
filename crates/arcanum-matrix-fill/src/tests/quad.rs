//! V-QUAD: Quadrature convergence validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const FREQ: f64 = 300e6;
const SEG_LENGTH: f64 = 0.05;
const A_MOD: f64 = 0.005;

/// V-QUAD-001 — Self-Impedance Quadrature Convergence.
///
/// Z[0,0] converges monotonically as near-singular quadrature order increases.
#[test]
fn v_quad_001_self_impedance_convergence() {
    let mesh = helpers::straight_wire_mesh(1, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut z_values = Vec::new();
    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_near_singular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        z_values.push(z.read(0, 0));
    }

    // Reference: highest order result.
    let z_ref = z_values.last().unwrap();

    // Relative convergence at p=32 vs p=64 should be < 1e-8.
    let z_32 = z_values[3]; // orders[3] = 32
    let rel_diff = (z_32 - *z_ref).abs() / z_ref.abs();
    assert!(
        rel_diff < 1e-8,
        "V-QUAD-001: |Z(p=32) - Z(p=64)| / |Z(p=64)| = {rel_diff:.2e}, expected < 1e-8"
    );

    // Monotonic convergence: differences from reference should decrease.
    let diffs: Vec<f64> = z_values.iter().map(|z| (*z - *z_ref).abs()).collect();
    for i in 1..diffs.len() - 1 {
        assert!(
            diffs[i] <= diffs[i - 1] + 1e-16,
            "V-QUAD-001: convergence not monotonic: diff[{}]={:.2e} > diff[{}]={:.2e}",
            i, diffs[i], i - 1, diffs[i - 1],
        );
    }
}

/// V-QUAD-002 — Near-Neighbor Mutual Impedance Quadrature Convergence.
///
/// Z[0,1] for adjacent segments converges to 8 significant figures at p=32.
#[test]
fn v_quad_002_near_neighbor_convergence() {
    let mesh = helpers::straight_wire_mesh(2, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut z_values = Vec::new();
    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_near_singular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        z_values.push(z.read(0, 1));
    }

    let z_ref = z_values.last().unwrap();

    let z_32 = z_values[3];
    let rel_diff = (z_32 - *z_ref).abs() / z_ref.abs();
    assert!(
        rel_diff < 1e-8,
        "V-QUAD-002: |Z(p=32) - Z(p=64)| / |Z(p=64)| = {rel_diff:.2e}, expected < 1e-8"
    );

    // Monotonic convergence.
    let diffs: Vec<f64> = z_values.iter().map(|z| (*z - *z_ref).abs()).collect();
    for i in 1..diffs.len() - 1 {
        assert!(
            diffs[i] <= diffs[i - 1] + 1e-16,
            "V-QUAD-002: convergence not monotonic: diff[{}]={:.2e} > diff[{}]={:.2e}",
            i, diffs[i], i - 1, diffs[i - 1],
        );
    }
}

/// V-QUAD-003 — Far Off-Diagonal Mutual Impedance Quadrature Convergence.
///
/// Z[0,10] for well-separated segments converges rapidly.
#[test]
fn v_quad_003_far_off_diagonal_convergence() {
    let mesh = helpers::straight_wire_mesh(11, SEG_LENGTH, A_MOD);
    let orders = [4, 8, 16, 32, 64];

    let mut z_values = Vec::new();
    for &order in &orders {
        let config = MatrixFillConfig {
            quadrature_order_regular: order,
            ..MatrixFillConfig::default()
        };
        let z = fill_impedance_matrix(&mesh, FREQ, &config);
        z_values.push(z.read(0, 10));
    }

    let z_ref = z_values.last().unwrap();

    let z_32 = z_values[3];
    let rel_diff = (z_32 - *z_ref).abs() / z_ref.abs();
    assert!(
        rel_diff < 1e-8,
        "V-QUAD-003: |Z(p=32) - Z(p=64)| / |Z(p=64)| = {rel_diff:.2e}, expected < 1e-8"
    );

    // Convergence for well-separated elements should be fast.
    // Once differences drop to machine epsilon, ordering is noise — use a floor.
    let diffs: Vec<f64> = z_values.iter().map(|z| (*z - *z_ref).abs()).collect();
    let eps_floor = 1e-14 * z_ref.abs(); // relative machine-epsilon floor
    for i in 1..diffs.len() - 1 {
        assert!(
            diffs[i] <= diffs[i - 1] + eps_floor,
            "V-QUAD-003: convergence not monotonic: diff[{}]={:.2e} > diff[{}]={:.2e}",
            i, diffs[i], i - 1, diffs[i - 1],
        );
    }
}
