use arcanum_geometry::mesh::Mesh;
use faer::complex_native::c64;
use std::f64::consts::PI;

use crate::config::MatrixFillConfig;
use crate::constants::{EPS_0, MU_0};
use crate::exact_kernel::evaluate_exact_kernel;
use crate::quadrature::QuadratureTables;

/// Compute the impedance matrix element Z[m,n] for a regular (well-separated)
/// element pair where |m-n| ≥ 2.
///
/// Uses product Gauss-Legendre quadrature at `config.quadrature_order_regular`
/// for the T1 (vector potential) term. The T2 (scalar potential) term reduces
/// to four endpoint evaluations of the kernel.
///
/// ```text
/// Z[m,n] = jωμ₀/(4π) × T1  -  1/(jωε₀·4π) × T2
///
/// T1 = ∫∫ [t̂_m·t̂_n] K_exact ds ds'                 (double integral, GL quadrature)
/// T2 = K(P_{m+1}, P_{n+1}) - K(P_{m+1}, P_n)        (four endpoint evaluations)
///    - K(P_m,     P_{n+1}) + K(P_m,     P_n)
/// ```
///
/// The T2 form comes from the double integral ∫∫ ∂²K/(∂s∂s') ds ds', which
/// collapses to endpoint evaluations when integrated analytically in both
/// coordinates (pulse basis + pulse testing). This form is manifestly symmetric
/// in m↔n since K is symmetric.
pub fn compute_regular(
    m: usize,
    n: usize,
    mesh: &Mesh,
    k: f64,
    quad_tables: &QuadratureTables,
    config: &MatrixFillConfig,
) -> c64 {
    let seg_m = &mesh.segments[m];
    let seg_n = &mesh.segments[n];

    let omega = k * crate::constants::C_LIGHT;
    let order = config.quadrature_order_regular;
    let gl = quad_tables.gl(order);
    let az_nw = quad_tables.azimuthal();

    let wire_radius_n = seg_n.wire_radius;

    // --- T1: double integral via product GL quadrature ---
    let mut t1 = c64::new(0.0, 0.0);

    // Map GL node ξ ∈ [-1,1] to σ ∈ [0,1]: σ = (ξ+1)/2, dσ/dξ = 1/2.
    // Arc length element: ds = |r'(σ)| dσ = |r'(σ)| × (1/2) dξ.

    for &(xi_m, w_m) in gl {
        let sigma_m = 0.5 * (xi_m + 1.0);
        let r_m = seg_m.curve.evaluate(sigma_m);
        let t_m_raw = seg_m.curve.tangent(sigma_m);
        let speed_m = t_m_raw.norm();
        let t_hat_m = t_m_raw / speed_m;

        // ds_m = speed_m × (1/2) × dξ_m
        let ds_m = speed_m * 0.5 * w_m;

        for &(xi_n, w_n) in gl {
            let sigma_n = 0.5 * (xi_n + 1.0);
            let r_n = seg_n.curve.evaluate(sigma_n);
            let t_n_raw = seg_n.curve.tangent(sigma_n);
            let speed_n = t_n_raw.norm();
            let t_hat_n = t_n_raw / speed_n;

            let ds_n = speed_n * 0.5 * w_n;

            let dot_tt = t_hat_m.dot(&t_hat_n);

            let k_exact =
                evaluate_exact_kernel(&r_m, &r_n, &t_hat_n, wire_radius_n, k, az_nw);

            t1 += k_exact * (dot_tt * ds_m * ds_n);
        }
    }

    // --- T2: four endpoint evaluations ---
    // T2 = K(P_{m+1}, P_{n+1}) - K(P_{m+1}, P_n) - K(P_m, P_{n+1}) + K(P_m, P_n)
    //
    // Each K evaluation uses the exact kernel with the source tangent and radius
    // at the source endpoint.
    let r_m_start = seg_m.start();
    let r_m_end = seg_m.end();
    let r_n_start = seg_n.start();
    let r_n_end = seg_n.end();

    let t_hat_n_start = seg_n.curve.tangent(0.0).normalize();
    let t_hat_n_end = seg_n.curve.tangent(1.0).normalize();

    let k_me_ne =
        evaluate_exact_kernel(&r_m_end, &r_n_end, &t_hat_n_end, wire_radius_n, k, az_nw);
    let k_me_ns =
        evaluate_exact_kernel(&r_m_end, &r_n_start, &t_hat_n_start, wire_radius_n, k, az_nw);
    let k_ms_ne =
        evaluate_exact_kernel(&r_m_start, &r_n_end, &t_hat_n_end, wire_radius_n, k, az_nw);
    let k_ms_ns =
        evaluate_exact_kernel(&r_m_start, &r_n_start, &t_hat_n_start, wire_radius_n, k, az_nw);

    let t2 = k_me_ne - k_me_ns - k_ms_ne + k_ms_ns;

    // Z[m,n] = jωμ₀/(4π) × T1 - 1/(jωε₀·4π) × T2
    let j = c64::new(0.0, 1.0);
    let coeff_t1 = j * omega * MU_0 / (4.0 * PI);
    let coeff_t2 = c64::new(1.0, 0.0) / (j * omega * EPS_0 * 4.0 * PI);

    coeff_t1 * t1 - coeff_t2 * t2
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quadrature::QuadratureTables;
    use arcanum_geometry::mesh::{
        CurveParams, GroundDescriptor, GroundType, LinearParams, Material, Segment, TagMap,
    };

    /// Build a minimal mesh of collinear segments along the z-axis.
    fn straight_wire_mesh(n_segments: usize, seg_length: f64, wire_radius: f64) -> Mesh {
        use nalgebra::Vector3;

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
    fn regular_element_is_finite_and_nonzero() {
        // 4-segment dipole, compute Z[0,2] (regular, |m-n|=2).
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let k = 2.0 * PI; // λ = 1 m
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z = compute_regular(0, 2, &mesh, k, &tables, &config);

        assert!(z.abs().is_finite(), "Z[0,2] should be finite");
        assert!(z.abs() > 0.0, "Z[0,2] should be nonzero");
    }

    #[test]
    fn regular_element_symmetry() {
        // Z[m,n] should equal Z[n,m] for regular elements.
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_02 = compute_regular(0, 2, &mesh, k, &tables, &config);
        let z_20 = compute_regular(2, 0, &mesh, k, &tables, &config);

        let diff = (z_02 - z_20).abs();
        let scale = z_02.abs().max(z_20.abs());
        assert!(
            diff / scale < 1e-10,
            "Z[0,2] and Z[2,0] should match: Z[0,2]={:?}, Z[2,0]={:?}, rel_diff={}",
            z_02,
            z_20,
            diff / scale,
        );
    }

    #[test]
    fn regular_element_decreases_with_distance() {
        // |Z[0,2]| > |Z[0,3]| for collinear segments of equal length.
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_02 = compute_regular(0, 2, &mesh, k, &tables, &config);
        let z_03 = compute_regular(0, 3, &mesh, k, &tables, &config);

        assert!(
            z_02.abs() > z_03.abs(),
            "|Z[0,2]| = {} should be > |Z[0,3]| = {}",
            z_02.abs(),
            z_03.abs(),
        );
    }
}
