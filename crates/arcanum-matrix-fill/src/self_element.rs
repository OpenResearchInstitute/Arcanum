use arcanum_geometry::mesh::Mesh;
use faer::complex_native::c64;
use std::f64::consts::PI;

use crate::config::MatrixFillConfig;
use crate::constants::{C_LIGHT, EPS_0, MU_0};
use crate::exact_kernel::evaluate_exact_kernel;
use crate::quadrature::QuadratureTables;

/// Compute the self-impedance matrix element Z[m,m].
///
/// The T1 (vector potential) term uses near-singular extraction:
///
/// ```text
/// T1[m,m] = I_near + ∫∫ [(t̂·t̂') K_exact - K_near] ds ds'
/// ```
///
/// where:
/// - `K_near(s,s') = 1/√(a² + (s-s')²)` captures the near-singular peak
/// - `I_near = 2[Δ sinh⁻¹(Δ/a) - √(a²+Δ²) + a]` is the analytic double integral
///   of K_near
/// - The smooth remainder `(t̂·t̂') K_exact - K_near` is integrated with GL
///   quadrature at `quadrature_order_near_singular`
///
/// The T2 (scalar potential) term is four endpoint evaluations, same as
/// for regular elements.
pub fn compute_self(
    m: usize,
    mesh: &Mesh,
    k: f64,
    quad_tables: &QuadratureTables,
    config: &MatrixFillConfig,
) -> c64 {
    let seg = &mesh.segments[m];
    let a = seg.wire_radius;
    let delta = seg.curve.arc_length();
    let omega = k * C_LIGHT;

    // --- T1: near-singular extraction ---

    // Analytic part: I_near = 2[Δ sinh⁻¹(Δ/a) - √(a²+Δ²) + a]
    let i_near = 2.0 * (delta * (delta / a).asinh() - (a * a + delta * delta).sqrt() + a);

    // Smooth remainder: ∫∫ [(t̂·t̂') K_exact - K_near] ds ds'
    // Near s ≈ s': (t̂·t̂') ≈ 1 and K_exact ≈ K_near, so the remainder → 0 (smooth).
    // Away from s ≈ s': both K_exact and K_near are smooth.
    let order = config.quadrature_order_near_singular;
    let gl = quad_tables.gl(order);
    let az_nw = quad_tables.azimuthal();

    let mut t1_smooth = c64::new(0.0, 0.0);

    for &(xi_m, w_m) in gl {
        let sigma_m = 0.5 * (xi_m + 1.0);
        let r_m = seg.curve.evaluate(sigma_m);
        let t_m_raw = seg.curve.tangent(sigma_m);
        let speed_m = t_m_raw.norm();
        let t_hat_m = t_m_raw / speed_m;
        let ds_m = speed_m * 0.5 * w_m;
        // Arc length coordinate (exact for straight/arc, approximate for helix).
        let s_m = sigma_m * delta;

        for &(xi_n, w_n) in gl {
            let sigma_n = 0.5 * (xi_n + 1.0);
            let r_n = seg.curve.evaluate(sigma_n);
            let t_n_raw = seg.curve.tangent(sigma_n);
            let speed_n = t_n_raw.norm();
            let t_hat_n = t_n_raw / speed_n;
            let ds_n = speed_n * 0.5 * w_n;
            let s_n = sigma_n * delta;

            let dot_tt = t_hat_m.dot(&t_hat_n);

            let k_exact = evaluate_exact_kernel(&r_m, &r_n, &t_hat_n, a, k, az_nw);

            let d = s_m - s_n;
            let k_near = 1.0 / (a * a + d * d).sqrt();

            let smooth = k_exact * dot_tt - c64::new(k_near, 0.0);
            t1_smooth += smooth * (ds_m * ds_n);
        }
    }

    let t1 = c64::new(i_near, 0.0) + t1_smooth;

    // --- T2: four endpoint evaluations ---
    // T2[m,m] = K(end,end) - K(end,start) - K(start,end) + K(start,start)
    let r_start = seg.start();
    let r_end = seg.end();
    let t_hat_start = seg.curve.tangent(0.0).normalize();
    let t_hat_end = seg.curve.tangent(1.0).normalize();

    let k_ss = evaluate_exact_kernel(&r_start, &r_start, &t_hat_start, a, k, az_nw);
    let k_ee = evaluate_exact_kernel(&r_end, &r_end, &t_hat_end, a, k, az_nw);
    let k_se = evaluate_exact_kernel(&r_start, &r_end, &t_hat_end, a, k, az_nw);
    let k_es = evaluate_exact_kernel(&r_end, &r_start, &t_hat_start, a, k, az_nw);

    let t2 = k_ee - k_es - k_se + k_ss;

    // Z[m,m] = jωμ₀/(4π) × T1 - 1/(jωε₀·4π) × T2
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
    fn self_element_is_finite_and_nonzero() {
        let mesh = straight_wire_mesh(1, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z = compute_self(0, &mesh, k, &tables, &config);

        assert!(z.abs().is_finite(), "Z[0,0] should be finite, got {:?}", z);
        assert!(z.abs() > 0.0, "Z[0,0] should be nonzero");
    }

    #[test]
    fn self_element_positive_real_part() {
        // The self-impedance real part (radiation resistance) should be positive.
        let mesh = straight_wire_mesh(1, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z = compute_self(0, &mesh, k, &tables, &config);

        assert!(
            z.re > 0.0,
            "Re(Z[0,0]) should be positive (radiation resistance), got {}",
            z.re,
        );
    }

    #[test]
    fn self_element_positive_imaginary_part() {
        // For electrically short segments (Δ << λ), the self-reactance is
        // positive (inductive), dominated by the jωμ₀ T1 term.
        let mesh = straight_wire_mesh(1, 0.05, 0.001); // Δ/λ = 0.05
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z = compute_self(0, &mesh, k, &tables, &config);

        assert!(
            z.im > 0.0,
            "Im(Z[0,0]) should be positive (inductive) for short segment, got {}",
            z.im,
        );
    }

    #[test]
    fn self_element_larger_than_regular() {
        // |Z[m,m]| should be larger than |Z[m,n]| for well-separated elements.
        let mesh = straight_wire_mesh(4, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z_self = compute_self(0, &mesh, k, &tables, &config);
        let z_reg = crate::regular::compute_regular(0, 2, &mesh, k, &tables, &config);

        assert!(
            z_self.abs() > z_reg.abs(),
            "|Z[0,0]| = {} should be > |Z[0,2]| = {}",
            z_self.abs(),
            z_reg.abs(),
        );
    }

    #[test]
    fn self_elements_equal_for_identical_segments() {
        // Z[0,0] should equal Z[1,1] for identical segments.
        let mesh = straight_wire_mesh(3, 0.05, 0.001);
        let k = 2.0 * PI;
        let tables = QuadratureTables::new(16);
        let config = MatrixFillConfig::default();

        let z0 = compute_self(0, &mesh, k, &tables, &config);
        let z1 = compute_self(1, &mesh, k, &tables, &config);

        let diff = (z0 - z1).abs();
        let scale = z0.abs().max(z1.abs());
        assert!(
            diff / scale < 1e-10,
            "Z[0,0] and Z[1,1] should match for identical segments: \
             Z[0,0]={:?}, Z[1,1]={:?}, rel_diff={}",
            z0,
            z1,
            diff / scale,
        );
    }

    #[test]
    fn i_near_thin_wire_limit() {
        // Verify I_near → 2Δ[ln(2Δ/a) - 1] for thin wires.
        let delta: f64 = 0.05;
        let a: f64 = 1e-6; // very thin wire
        let i_near = 2.0 * (delta * (delta / a).asinh() - (a * a + delta * delta).sqrt() + a);
        let i_thin = 2.0 * delta * ((2.0 * delta / a).ln() - 1.0);

        let rel_err = (i_near - i_thin).abs() / i_thin.abs();
        assert!(
            rel_err < 1e-4,
            "I_near should converge to thin-wire limit: I_near={}, I_thin={}, rel_err={}",
            i_near,
            i_thin,
            rel_err,
        );
    }
}
