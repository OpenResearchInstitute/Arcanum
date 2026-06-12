/// Configuration for impedance matrix fill.
///
/// Controls quadrature orders and accuracy thresholds. Default values are
/// conservative, prioritizing accuracy over speed. Use [`MatrixFillConfig::fast`]
/// for exploratory runs where speed matters more than precision.
#[derive(Debug, Clone)]
pub struct MatrixFillConfig {
    /// Gauss-Legendre order for regular (well-separated) element pairs.
    pub quadrature_order_regular: usize,
    /// Gauss-Legendre order for self and near-neighbor elements where the
    /// integrand is near-singular.
    pub quadrature_order_near_singular: usize,
    /// Number of azimuthal integration points for the exact cylindrical kernel.
    pub quadrature_order_azimuthal: usize,
    /// Relative convergence threshold for adaptive quadrature.
    pub convergence_threshold: f64,
    /// Distance (in multiples of wire radius) below which element pairs are
    /// treated as near-singular rather than regular.
    pub near_singular_distance_ratio: f64,
}

impl Default for MatrixFillConfig {
    fn default() -> Self {
        Self {
            quadrature_order_regular: 8,
            quadrature_order_near_singular: 32,
            quadrature_order_azimuthal: 16,
            convergence_threshold: 1e-10,
            near_singular_distance_ratio: 3.0,
        }
    }
}

impl MatrixFillConfig {
    /// Preset with relaxed thresholds for fast exploratory runs.
    ///
    /// Reduces quadrature orders and loosens the convergence threshold.
    /// Not recommended for production simulations.
    pub fn fast() -> Self {
        Self {
            quadrature_order_regular: 4,
            quadrature_order_near_singular: 16,
            quadrature_order_azimuthal: 8,
            convergence_threshold: 1e-6,
            near_singular_distance_ratio: 3.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values() {
        let cfg = MatrixFillConfig::default();
        assert_eq!(cfg.quadrature_order_regular, 8);
        assert_eq!(cfg.quadrature_order_near_singular, 32);
        assert_eq!(cfg.quadrature_order_azimuthal, 16);
        assert!((cfg.convergence_threshold - 1e-10).abs() < f64::EPSILON);
        assert!((cfg.near_singular_distance_ratio - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn fast_preset_is_less_expensive() {
        let default = MatrixFillConfig::default();
        let fast = MatrixFillConfig::fast();
        assert!(fast.quadrature_order_regular < default.quadrature_order_regular);
        assert!(fast.quadrature_order_near_singular < default.quadrature_order_near_singular);
        assert!(fast.quadrature_order_azimuthal < default.quadrature_order_azimuthal);
        assert!(fast.convergence_threshold > default.convergence_threshold);
    }
}
