use std::collections::HashMap;
use std::f64::consts::PI;
use std::num::NonZeroUsize;

use gauss_quad::GaussLegendre;

/// Precomputed Gauss-Legendre nodes and weights for all quadrature orders
/// used during the matrix fill.
///
/// GL tables are computed once at the start of a fill and shared (read-only)
/// across all parallel workers. Nodes are on the standard interval \[-1, 1\].
///
/// Azimuthal nodes and weights are mapped to \[0, 2π\] for exact kernel
/// surface integration.
pub struct QuadratureTables {
    /// GL nodes and weights by order. Each entry is a `Vec<(node, weight)>`
    /// on \[-1, 1\].
    gl: HashMap<usize, Vec<(f64, f64)>>,
    /// Azimuthal nodes and weights on \[0, 2π\].
    azimuthal: Vec<(f64, f64)>,
}

/// The set of GL orders we precompute.
const PRECOMPUTED_ORDERS: &[usize] = &[4, 8, 16, 32, 64];

impl QuadratureTables {
    /// Build tables for all standard orders plus the azimuthal order from
    /// `config`.
    pub fn new(azimuthal_order: usize) -> Self {
        let mut gl = HashMap::new();
        for &order in PRECOMPUTED_ORDERS {
            let rule = GaussLegendre::new(
                NonZeroUsize::new(order).expect("order must be nonzero"),
            );
            let pairs: Vec<(f64, f64)> = rule
                .as_node_weight_pairs()
                .to_vec();
            gl.insert(order, pairs);
        }

        // Azimuthal nodes: map GL nodes from [-1, 1] to [0, 2π].
        // φ = π(ξ + 1), dφ = π dξ
        let az_rule = GaussLegendre::new(
            NonZeroUsize::new(azimuthal_order).expect("azimuthal order must be nonzero"),
        );
        let azimuthal: Vec<(f64, f64)> = az_rule
            .as_node_weight_pairs()
            .iter()
            .map(|&(node, weight)| {
                let phi = PI * (node + 1.0); // maps [-1,1] → [0, 2π]
                let w = PI * weight; // Jacobian of the affine map
                (phi, w)
            })
            .collect();

        Self { gl, azimuthal }
    }

    /// Get GL nodes/weights for the given order on \[-1, 1\].
    ///
    /// Panics if `order` is not one of {4, 8, 16, 32, 64}.
    pub fn gl(&self, order: usize) -> &[(f64, f64)] {
        self.gl
            .get(&order)
            .unwrap_or_else(|| panic!("GL order {order} not precomputed"))
    }

    /// Azimuthal nodes (φ) and weights on \[0, 2π\].
    pub fn azimuthal(&self) -> &[(f64, f64)] {
        &self.azimuthal
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gl_weights_sum_to_two() {
        let tables = QuadratureTables::new(16);
        for &order in PRECOMPUTED_ORDERS {
            let sum: f64 = tables.gl(order).iter().map(|&(_, w)| w).sum();
            assert!(
                (sum - 2.0).abs() < 1e-14,
                "order {order}: weight sum = {sum}, expected 2.0"
            );
        }
    }

    #[test]
    fn gl_integrates_x_squared() {
        // ∫_{-1}^{1} x² dx = 2/3. Exact for any order ≥ 2.
        let tables = QuadratureTables::new(16);
        for &order in PRECOMPUTED_ORDERS {
            let integral: f64 = tables
                .gl(order)
                .iter()
                .map(|&(x, w)| w * x * x)
                .sum();
            assert!(
                (integral - 2.0 / 3.0).abs() < 1e-14,
                "order {order}: ∫x² dx = {integral}, expected 2/3"
            );
        }
    }

    #[test]
    fn azimuthal_weights_sum_to_two_pi() {
        let tables = QuadratureTables::new(16);
        let sum: f64 = tables.azimuthal().iter().map(|&(_, w)| w).sum();
        assert!(
            (sum - 2.0 * PI).abs() < 1e-14,
            "azimuthal weight sum = {sum}, expected 2π"
        );
    }

    #[test]
    fn azimuthal_nodes_in_range() {
        let tables = QuadratureTables::new(16);
        for &(phi, _) in tables.azimuthal() {
            assert!(phi >= 0.0 && phi <= 2.0 * PI, "φ = {phi} out of [0, 2π]");
        }
    }

    #[test]
    fn correct_number_of_points() {
        let tables = QuadratureTables::new(8);
        for &order in PRECOMPUTED_ORDERS {
            assert_eq!(tables.gl(order).len(), order);
        }
        assert_eq!(tables.azimuthal().len(), 8);
    }

    #[test]
    #[should_panic(expected = "not precomputed")]
    fn unknown_order_panics() {
        let tables = QuadratureTables::new(16);
        tables.gl(7); // not in {4,8,16,32,64}
    }
}
