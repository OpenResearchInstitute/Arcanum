use faer::complex_native::c64;
use nalgebra::Vector3;

/// Construct a perpendicular frame (n̂_r, n̂_φ) from a unit tangent vector t̂.
///
/// Uses the minimum-component Gram-Schmidt construction:
/// 1. Choose the standard basis vector v least aligned with t̂ (the one whose
///    component |t̂·v| is smallest).
/// 2. Gram-Schmidt: n̂_r = normalize(v − (v·t̂) t̂)
/// 3. Right-hand cross: n̂_φ = t̂ × n̂_r
///
/// This is well-defined for all unit vectors t̂ — including straight segments
/// where the Frenet-Serret frame is undefined — and guarantees
/// |v − (v·t̂)t̂| ≥ √(2/3) > 0.
pub fn perpendicular_frame(t_hat: &Vector3<f64>) -> (Vector3<f64>, Vector3<f64>) {
    // Pick the standard basis vector whose component along t̂ is smallest.
    let abs_x = t_hat.x.abs();
    let abs_y = t_hat.y.abs();
    let abs_z = t_hat.z.abs();

    let v = if abs_x <= abs_y && abs_x <= abs_z {
        Vector3::x()
    } else if abs_y <= abs_z {
        Vector3::y()
    } else {
        Vector3::z()
    };

    // Gram-Schmidt: remove the component of v along t̂.
    let proj = v - t_hat * v.dot(t_hat);
    let n_r = proj.normalize();

    // Right-hand perpendicular.
    let n_phi = t_hat.cross(&n_r);

    (n_r, n_phi)
}

/// Evaluate the exact cylindrical kernel K_exact at a single (observation, source)
/// point pair by azimuthal integration over the wire surface.
///
/// ```text
/// K_exact = 1/(2π) ∫₀^{2π} G₀(r_obs, r_surf(φ)) dφ
/// ```
///
/// where `r_surf(φ) = r_axis + a [n̂_r cos(φ) + n̂_φ sin(φ)]` and
/// `G₀(r, r') = e^{-jkR} / R`.
///
/// The azimuthal quadrature nodes and weights (on \[0, 2π\]) are provided by
/// the caller via `az_nw`. The 1/(2π) normalization is included.
///
/// # Arguments
///
/// * `r_obs` — observation point (on segment m axis)
/// * `r_axis` — source point on segment n axis
/// * `t_hat` — unit tangent at the source point
/// * `wire_radius` — radius *a* of the source wire
/// * `k` — free-space wavenumber (rad/m)
/// * `az_nw` — azimuthal (φ, weight) pairs on \[0, 2π\]
pub fn evaluate_exact_kernel(
    r_obs: &Vector3<f64>,
    r_axis: &Vector3<f64>,
    t_hat: &Vector3<f64>,
    wire_radius: f64,
    k: f64,
    az_nw: &[(f64, f64)],
) -> c64 {
    let (n_r, n_phi) = perpendicular_frame(t_hat);

    let mut sum = c64::new(0.0, 0.0);

    for &(phi, w) in az_nw {
        let (sin_phi, cos_phi) = phi.sin_cos();
        // Surface point: r_axis + a (n̂_r cos φ + n̂_φ sin φ)
        let r_surf = r_axis + wire_radius * (n_r * cos_phi + n_phi * sin_phi);
        let diff = r_obs - r_surf;
        let r_dist = diff.norm();

        // G₀ = e^{-jkR} / R
        let phase = -k * r_dist;
        let g0 = c64::cis(phase) * (1.0 / r_dist);

        sum += g0 * w;
    }

    // Normalize by 1/(2π)
    use std::f64::consts::PI;
    sum * (1.0 / (2.0 * PI))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn make_azimuthal_nodes(n: usize) -> Vec<(f64, f64)> {
        // Simple uniform trapezoidal rule on [0, 2π] for testing.
        let dw = 2.0 * PI / n as f64;
        (0..n)
            .map(|i| (dw * (i as f64 + 0.5), dw))
            .collect()
    }

    #[test]
    fn frame_orthogonality_z_axis() {
        let t = Vector3::new(0.0, 0.0, 1.0);
        let (n_r, n_phi) = perpendicular_frame(&t);

        assert!((n_r.dot(&t)).abs() < 1e-15, "n_r not perpendicular to t");
        assert!((n_phi.dot(&t)).abs() < 1e-15, "n_phi not perpendicular to t");
        assert!((n_r.dot(&n_phi)).abs() < 1e-15, "n_r not perpendicular to n_phi");
        assert!((n_r.norm() - 1.0).abs() < 1e-15, "n_r not unit");
        assert!((n_phi.norm() - 1.0).abs() < 1e-15, "n_phi not unit");
    }

    #[test]
    fn frame_orthogonality_arbitrary() {
        let directions = [
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0).normalize(),
            Vector3::new(1.0, 1.0, 1.0).normalize(),
            Vector3::new(-0.3, 0.7, 0.5).normalize(),
        ];

        for t in &directions {
            let (n_r, n_phi) = perpendicular_frame(t);

            assert!(
                n_r.dot(t).abs() < 1e-14,
                "n_r·t = {} for t = {:?}",
                n_r.dot(t),
                t
            );
            assert!(
                n_phi.dot(t).abs() < 1e-14,
                "n_phi·t = {} for t = {:?}",
                n_phi.dot(t),
                t
            );
            assert!(
                n_r.dot(&n_phi).abs() < 1e-14,
                "n_r·n_phi = {} for t = {:?}",
                n_r.dot(&n_phi),
                t
            );
            assert!(
                (n_r.norm() - 1.0).abs() < 1e-14,
                "n_r not unit for t = {:?}",
                t
            );
            assert!(
                (n_phi.norm() - 1.0).abs() < 1e-14,
                "n_phi not unit for t = {:?}",
                t
            );
        }
    }

    #[test]
    fn frame_right_hand_rule() {
        // n̂_φ = t̂ × n̂_r, so t̂ · (n̂_r × n̂_φ) should be +1 (right-handed).
        let t = Vector3::new(0.3, -0.5, 0.8).normalize();
        let (n_r, n_phi) = perpendicular_frame(&t);
        let triple = t.dot(&n_r.cross(&n_phi));
        assert!(
            (triple - 1.0).abs() < 1e-14,
            "frame not right-handed: triple product = {triple}"
        );
    }

    #[test]
    fn kernel_approaches_thin_wire_for_large_separation() {
        // When the observation point is far from the source, the exact kernel
        // should approach the thin-wire (axis) Green's function G₀(R_axis).
        let r_obs = Vector3::new(0.0, 0.0, 100.0);
        let r_axis = Vector3::new(0.0, 0.0, 0.0);
        let t_hat = Vector3::new(0.0, 0.0, 1.0);
        let wire_radius = 0.001; // 1 mm wire, 100 m away
        let k = 2.0 * PI; // λ = 1 m

        let az_nw = make_azimuthal_nodes(32);
        let k_exact = evaluate_exact_kernel(&r_obs, &r_axis, &t_hat, wire_radius, k, &az_nw);

        // Thin-wire reference: G₀ = e^{-jkR}/R with R = 100.
        let r_dist = 100.0;
        let g0_thin = c64::cis(-k * r_dist) * (1.0 / r_dist);

        let rel_err = (k_exact - g0_thin).abs() / g0_thin.abs();
        assert!(
            rel_err < 1e-6,
            "exact kernel should match thin-wire at large distance, rel_err = {rel_err}"
        );
    }

    #[test]
    fn kernel_nonzero_for_self_point() {
        // When r_obs is on the axis and r_axis is the same point, the exact
        // kernel is finite (regularized by the wire radius).
        let r_obs = Vector3::new(0.0, 0.0, 0.0);
        let r_axis = Vector3::new(0.0, 0.0, 0.0);
        let t_hat = Vector3::new(0.0, 0.0, 1.0);
        let wire_radius = 0.01;
        let k = 2.0 * PI;

        let az_nw = make_azimuthal_nodes(32);
        let k_exact = evaluate_exact_kernel(&r_obs, &r_axis, &t_hat, wire_radius, k, &az_nw);

        // Should be finite and nonzero.
        assert!(k_exact.abs() > 0.0, "kernel should be nonzero");
        assert!(k_exact.abs().is_finite(), "kernel should be finite");

        // At s=s', all surface points are at distance a from the axis, so
        // K_exact ≈ e^{-jka}/a.
        let expected_mag = 1.0 / wire_radius;
        let rel_err = (k_exact.abs() - expected_mag).abs() / expected_mag;
        assert!(
            rel_err < 0.01,
            "magnitude should be ≈ 1/a, got {}, expected {}, rel_err = {}",
            k_exact.abs(),
            expected_mag,
            rel_err,
        );
    }
}
