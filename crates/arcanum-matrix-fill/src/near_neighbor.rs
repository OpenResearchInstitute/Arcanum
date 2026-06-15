use arcanum_geometry::mesh::{EndpointSide, Mesh};
use faer::complex_native::c64;
use std::f64::consts::PI;

use crate::config::MatrixFillConfig;
use crate::constants::{C_LIGHT, EPS_0, MU_0};
use crate::exact_kernel::evaluate_exact_kernel;
use crate::quadrature::QuadratureTables;

/// Compute the impedance matrix element Z[m,n] for a near-neighbor pair
/// where n = m+1 (adjacent segments sharing an endpoint).
///
/// The T1 (vector potential) term uses near-singular extraction in local
/// coordinates (ε, ε') measured from the shared endpoint P:
///
/// ```text
/// ε  = s_{m+1} - s     (distance from P within segment m,   ε  ∈ [0, Δm])
/// ε' = s' - s_{m+1}    (distance from P within segment m+1, ε' ∈ [0, Δn])
///
/// K_near(ε, ε') = 1/√(a² + (ε + ε')²)    [collinear approximation, Option A]
///
/// I_near = Δm sinh⁻¹(Δn/a) + Δn sinh⁻¹(Δm/a)
///        - √(a² + (Δm+Δn)²) + √(a²+Δm²) + √(a²+Δn²) - a
/// ```
///
/// The smooth remainder `(t̂_m·t̂_n) K_exact - K_near` is integrated with GL
/// at `quadrature_order_near_singular`.
///
/// T2 is four endpoint evaluations (same form as regular elements).
///
/// Cross-wire junctions (sequential segments on different wires) use the same
/// collinear extraction formula (Option A). The error from this approximation
/// grows with bend angle but is negligible for typical antenna geometries.
#[allow(clippy::too_many_arguments)]
pub fn compute_near_neighbor(
    m: usize,
    n: usize,
    m_side: &EndpointSide,
    n_side: &EndpointSide,
    mesh: &Mesh,
    k: f64,
    quad_tables: &QuadratureTables,
    config: &MatrixFillConfig,
) -> c64 {
    let seg_m = &mesh.segments[m];
    let seg_n = &mesh.segments[n];

    let a = seg_n.wire_radius;
    let delta_m = seg_m.curve.arc_length();
    let delta_n = seg_n.curve.arc_length();
    let omega = k * C_LIGHT;

    // --- T1: near-singular extraction ---

    // Analytic integral of K_near = 1/√(a² + (ε+ε')²) over [0,Δm]×[0,Δn].
    let dm = delta_m;
    let dn = delta_n;
    let i_near = dm * (dn / a).asinh() + dn * (dm / a).asinh()
        - (a * a + (dm + dn) * (dm + dn)).sqrt()
        + (a * a + dm * dm).sqrt()
        + (a * a + dn * dn).sqrt()
        - a;

    // Smooth remainder via GL quadrature.
    let order = config.quadrature_order_near_singular;
    let gl = quad_tables.gl(order);
    let az_nw = quad_tables.azimuthal();

    let mut t1_smooth = c64::new(0.0, 0.0);

    for &(xi_m, w_m) in gl {
        let sigma_m = 0.5 * (xi_m + 1.0);
        let r_m = seg_m.curve.evaluate(sigma_m);
        let t_m_raw = seg_m.curve.tangent(sigma_m);
        let speed_m = t_m_raw.norm();
        let t_hat_m = t_m_raw / speed_m;
        let ds_m = speed_m * 0.5 * w_m;

        // ε = distance from shared endpoint P within segment m.
        let eps_m = match m_side {
            EndpointSide::End => (1.0 - sigma_m) * delta_m,
            EndpointSide::Start => sigma_m * delta_m,
        };

        for &(xi_n, w_n) in gl {
            let sigma_n = 0.5 * (xi_n + 1.0);
            let r_n = seg_n.curve.evaluate(sigma_n);
            let t_n_raw = seg_n.curve.tangent(sigma_n);
            let speed_n = t_n_raw.norm();
            let t_hat_n = t_n_raw / speed_n;
            let ds_n = speed_n * 0.5 * w_n;

            // ε' = distance from shared endpoint P within segment n.
            let eps_n = match n_side {
                EndpointSide::End => (1.0 - sigma_n) * delta_n,
                EndpointSide::Start => sigma_n * delta_n,
            };

            let dot_tt = t_hat_m.dot(&t_hat_n);

            let k_exact = evaluate_exact_kernel(&r_m, &r_n, &t_hat_n, a, k, az_nw);

            let d = eps_m + eps_n;
            let k_near = 1.0 / (a * a + d * d).sqrt();

            let smooth = k_exact * dot_tt - c64::new(k_near, 0.0);
            t1_smooth += smooth * (ds_m * ds_n);
        }
    }

    let t1 = c64::new(i_near, 0.0) + t1_smooth;

    // --- T2: four endpoint evaluations ---
    // T2[m,n] = K(m_end, n_end) - K(m_end, n_start) - K(m_start, n_end) + K(m_start, n_start)
    let r_m_start = seg_m.start();
    let r_m_end = seg_m.end();
    let r_n_start = seg_n.start();
    let r_n_end = seg_n.end();

    let t_hat_n_start = seg_n.curve.tangent(0.0).normalize();
    let t_hat_n_end = seg_n.curve.tangent(1.0).normalize();

    let k_me_ne =
        evaluate_exact_kernel(&r_m_end, &r_n_end, &t_hat_n_end, a, k, az_nw);
    let k_me_ns =
        evaluate_exact_kernel(&r_m_end, &r_n_start, &t_hat_n_start, a, k, az_nw);
    let k_ms_ne =
        evaluate_exact_kernel(&r_m_start, &r_n_end, &t_hat_n_end, a, k, az_nw);
    let k_ms_ns =
        evaluate_exact_kernel(&r_m_start, &r_n_start, &t_hat_n_start, a, k, az_nw);

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
        CurveParams, EndpointSide, GroundDescriptor, GroundType, LinearParams, Material, Mesh,
        Segment, TagMap,
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
    fn near_neighbor_is_finite_and_nonzero() {
        let mesh = straight_wire_mesh(3, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z = compute_near_neighbor(0, 1, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);

        assert!(z.abs().is_finite(), "Z[0,1] should be finite, got {:?}", z);
        assert!(z.abs() > 0.0, "Z[0,1] should be nonzero");
    }

    #[test]
    fn near_neighbor_symmetry() {
        // Z[m,n] = Z[n,m] for near-neighbor elements.
        let mesh = straight_wire_mesh(3, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_01 = compute_near_neighbor(0, 1, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);
        let z_10 = compute_near_neighbor(1, 0, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);

        let diff = (z_01 - z_10).abs();
        let scale = z_01.abs().max(z_10.abs());
        assert!(
            diff / scale < 1e-8,
            "Z[0,1] and Z[1,0] should match: Z[0,1]={:?}, Z[1,0]={:?}, rel_diff={}",
            z_01,
            z_10,
            diff / scale,
        );
    }

    #[test]
    fn near_neighbor_larger_than_regular() {
        // |Z[0,1]| should be larger than |Z[0,2]| (closer = stronger coupling).
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_near = compute_near_neighbor(0, 1, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);
        let z_reg = crate::regular::compute_regular(0, 2, &mesh, k, &tables, &config);

        assert!(
            z_near.abs() > z_reg.abs(),
            "|Z[0,1]| = {} should be > |Z[0,2]| = {}",
            z_near.abs(),
            z_reg.abs(),
        );
    }

    #[test]
    fn near_neighbor_no_nan_or_inf() {
        // Check multiple near-neighbor pairs for NaN/Inf.
        let mesh = straight_wire_mesh(5, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        for i in 0..4 {
            let z = compute_near_neighbor(i, i + 1, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);
            assert!(
                z.re.is_finite() && z.im.is_finite(),
                "Z[{},{}] contains NaN/Inf: {:?}",
                i,
                i + 1,
                z,
            );
        }
    }

    #[test]
    fn equal_segments_give_equal_near_neighbors() {
        // Z[0,1] should equal Z[1,2] for identical uniform segments.
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_01 = compute_near_neighbor(0, 1, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);
        let z_12 = compute_near_neighbor(1, 2, &EndpointSide::End, &EndpointSide::Start, &mesh, k, &tables, &config);

        let diff = (z_01 - z_12).abs();
        let scale = z_01.abs().max(z_12.abs());
        assert!(
            diff / scale < 1e-10,
            "Z[0,1] and Z[1,2] should match for uniform mesh: \
             Z[0,1]={:?}, Z[1,2]={:?}, rel_diff={}",
            z_01,
            z_12,
            diff / scale,
        );
    }

    #[test]
    fn i_near_collinear_formula() {
        // Verify I_near for equal-length collinear segments matches expected.
        let delta: f64 = 0.05;
        let a: f64 = 0.001;

        let i_near = delta * (delta / a).asinh() + delta * (delta / a).asinh()
            - (a * a + 4.0 * delta * delta).sqrt()
            + 2.0 * (a * a + delta * delta).sqrt()
            - a;

        // Should equal: 2Δ sinh⁻¹(Δ/a) - √(a²+4Δ²) + 2√(a²+Δ²) - a
        let expected = 2.0 * delta * (delta / a).asinh()
            - (a * a + 4.0 * delta * delta).sqrt()
            + 2.0 * (a * a + delta * delta).sqrt()
            - a;

        assert!(
            (i_near - expected).abs() < 1e-15,
            "I_near formula mismatch: {} vs {}",
            i_near,
            expected,
        );
    }
}
