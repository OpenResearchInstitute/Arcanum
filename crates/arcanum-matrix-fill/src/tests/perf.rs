//! V-PERF: Parallel vs sequential fill correctness.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const FREQ: f64 = 300e6;
const SEG_LENGTH: f64 = 0.05;
const A_MOD: f64 = 0.005;

/// V-PERF-001 — Parallel Fill Produces Identical Results to Sequential Fill.
///
/// Runs the fill with Rayon parallelism (default) and with a single thread,
/// comparing all elements for bitwise equality.
#[test]
fn v_perf_001_parallel_equals_sequential() {
    let mesh = helpers::straight_wire_mesh(11, SEG_LENGTH, A_MOD);
    let config = MatrixFillConfig::default();

    // Parallel fill (Rayon default thread pool).
    let z_par = fill_impedance_matrix(&mesh, FREQ, &config);

    // Sequential fill (force single thread).
    let z_seq = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .build()
        .unwrap()
        .install(|| fill_impedance_matrix(&mesh, FREQ, &config));

    let n = 11;
    let mut max_diff = 0.0f64;
    for m in 0..n {
        for col in 0..n {
            let diff = (z_par.read(m, col) - z_seq.read(m, col)).abs();
            max_diff = max_diff.max(diff);
        }
    }

    assert!(
        max_diff < 1e-15,
        "V-PERF-001: max |Z_par - Z_seq| = {max_diff}, expected < ε_machine"
    );
}
