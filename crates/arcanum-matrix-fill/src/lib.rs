// arcanum-matrix-fill — Phase 2: Impedance Matrix Fill
//
// Computes the N×N complex impedance matrix Z[m,n] via the exact kernel
// Green's function integral evaluated with adaptive Gauss-Legendre quadrature.
// Parallelized over matrix elements using Rayon.

mod classify;
mod config;
mod constants;
mod exact_kernel;
mod near_neighbor;
mod quadrature;
mod regular;
mod self_element;
#[cfg(test)]
mod tests;
mod zmatrix;

pub use config::MatrixFillConfig;
pub use zmatrix::ZMatrix;

use arcanum_geometry::mesh::Mesh;
use rayon::prelude::*;

use classify::classify;
use constants::wavenumber;
use near_neighbor::compute_near_neighbor;
use quadrature::QuadratureTables;
use regular::compute_regular;
use self_element::compute_self;

/// Compute the N×N complex impedance matrix Z[m,n] for the given mesh and
/// frequency.
///
/// # Pipeline
///
/// 1. Precompute Gauss-Legendre quadrature tables (shared read-only).
/// 2. Classify upper-triangle (m,n) pairs into self / near-neighbor / regular.
/// 3. Fill all three categories in parallel via Rayon.
/// 4. Copy upper triangle to lower triangle (Z is symmetric).
///
/// # Arguments
///
/// - `mesh` — segment mesh from Phase 1. Borrowed immutably.
/// - `frequency` — simulation frequency in **Hz**.
/// - `config` — quadrature orders and accuracy thresholds.
pub fn fill_impedance_matrix(
    mesh: &Mesh,
    frequency: f64,
    config: &MatrixFillConfig,
) -> ZMatrix {
    let n = mesh.segments.len();
    let k = wavenumber(frequency);
    let quad_tables = QuadratureTables::new(config.quadrature_order_azimuthal);
    let classification = classify(n);
    let matrix = ZMatrix::new(n);

    // Step 3: Parallel fill — each element writes to a unique (row, col) cell.
    //
    // SAFETY: The classify function guarantees non-overlapping (m,n) pairs across
    // all three lists. Self elements write to (m,m), near-neighbor and regular
    // elements write to distinct upper-triangle cells. No two threads write the
    // same cell.

    classification.self_elements.par_iter().for_each(|&m| {
        let z = compute_self(m, mesh, k, &quad_tables, config);
        unsafe { matrix.write(m, m, z) };
    });

    classification
        .near_neighbor_elements
        .par_iter()
        .for_each(|&(m, n_idx)| {
            let z = compute_near_neighbor(m, n_idx, mesh, k, &quad_tables, config);
            unsafe { matrix.write(m, n_idx, z) };
        });

    classification
        .regular_elements
        .par_iter()
        .for_each(|&(m, n_idx)| {
            let z = compute_regular(m, n_idx, mesh, k, &quad_tables, config);
            unsafe { matrix.write(m, n_idx, z) };
        });

    // Step 4: Symmetry copy — Z[n,m] ← Z[m,n] for all m < n.
    for m in 0..n {
        for j in (m + 1)..n {
            let val = matrix.read(m, j);
            unsafe { matrix.write(j, m, val) };
        }
    }

    matrix
}

#[cfg(test)]
mod entry_tests {
    use super::*;
    use arcanum_geometry::mesh::{
        CurveParams, GroundDescriptor, GroundType, LinearParams, Material, Mesh, Segment, TagMap,
    };
    use nalgebra::Vector3;

    fn straight_wire_mesh(n_segments: usize, seg_length: f64, wire_radius: f64) -> Mesh {
        let segments: Vec<Segment> = (0..n_segments)
            .map(|i| {
                let z0 = i as f64 * seg_length;
                let z1 = z0 + seg_length;
                Segment {
                    curve: CurveParams::Linear(LinearParams {
                        start: Vector3::new(0.0, 0.0, z0),
                        end: Vector3::new(0.0, 0.0, z1),
                    }),
                    wire_radius,
                    material: Material::PEC,
                    tag: 1,
                    segment_index: i,
                    wire_index: 0,
                    is_image: false,
                }
            })
            .collect();

        Mesh {
            segments,
            junctions: vec![],
            endpoint_junction: vec![],
            ground: GroundDescriptor {
                ground_type: GroundType::None,
                conductivity: None,
                permittivity: None,
                images_generated: false,
            },
            tag_map: TagMap::default(),
        }
    }

    #[test]
    fn fill_returns_correct_dimensions() {
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);
        assert_eq!(z.n_segments(), 5);
    }

    #[test]
    fn fill_matrix_is_symmetric() {
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        for m in 0..5 {
            for n in 0..5 {
                let zmn = z.read(m, n);
                let znm = z.read(n, m);
                let diff = (zmn - znm).abs();
                assert!(
                    diff < 1e-15,
                    "Z[{},{}] != Z[{},{}]: {:?} vs {:?}",
                    m, n, n, m, zmn, znm,
                );
            }
        }
    }

    #[test]
    fn fill_no_nan_or_inf() {
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        for m in 0..5 {
            for n in 0..5 {
                let val = z.read(m, n);
                assert!(
                    val.re.is_finite() && val.im.is_finite(),
                    "Z[{},{}] is NaN/Inf: {:?}",
                    m, n, val,
                );
            }
        }
    }

    #[test]
    fn fill_diagonal_dominance() {
        // For a straight wire, diagonal elements should have the largest magnitude
        // in their row.
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        for m in 0..5 {
            let diag = z.read(m, m).abs();
            for n in 0..5 {
                if n != m {
                    assert!(
                        diag > z.read(m, n).abs(),
                        "|Z[{0},{0}]| = {1} should be > |Z[{0},{2}]| = {3}",
                        m, diag, n, z.read(m, n).abs(),
                    );
                }
            }
        }
    }

    #[test]
    fn fill_single_segment() {
        // A 1-segment mesh should produce a 1×1 matrix with just the self element.
        let mesh = straight_wire_mesh(1, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        assert_eq!(z.n_segments(), 1);
        let val = z.read(0, 0);
        assert!(val.abs() > 0.0, "Z[0,0] should be nonzero");
        assert!(val.re > 0.0, "Re(Z[0,0]) should be positive");
    }

    #[test]
    fn fill_uniform_mesh_toeplitz() {
        // For a uniform straight wire, Z should be approximately Toeplitz:
        // Z[m,n] depends only on |m-n|.
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let config = MatrixFillConfig::default();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        // Check that Z[0,2] ≈ Z[1,3] ≈ Z[2,4] (all have |m-n|=2).
        let z02 = z.read(0, 2);
        let z13 = z.read(1, 3);
        let z24 = z.read(2, 4);

        let scale = z02.abs();
        assert!(
            (z02 - z13).abs() / scale < 1e-10,
            "Z[0,2] and Z[1,3] should match: {:?} vs {:?}",
            z02, z13,
        );
        assert!(
            (z02 - z24).abs() / scale < 1e-10,
            "Z[0,2] and Z[2,4] should match: {:?} vs {:?}",
            z02, z24,
        );
    }

    #[test]
    fn fill_with_fast_config() {
        // The fast preset should also produce a valid (finite, symmetric) matrix.
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let config = MatrixFillConfig::fast();
        let z = fill_impedance_matrix(&mesh, 300e6, &config);

        for m in 0..4 {
            for n in 0..4 {
                let val = z.read(m, n);
                assert!(val.re.is_finite() && val.im.is_finite());
                let diff = (val - z.read(n, m)).abs();
                assert!(diff < 1e-15);
            }
        }
    }
}
