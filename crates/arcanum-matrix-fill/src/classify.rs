/// Result of classifying all upper-triangle element pairs into integration
/// categories.
///
/// Every (m, n) pair with m ≤ n is classified as self, near-neighbor, or
/// regular. Only the upper triangle is computed; the lower triangle is filled
/// by symmetry copy after the parallel fill.
pub struct ElementClassification {
    /// Diagonal indices: m = n. Each entry is a segment index.
    pub self_elements: Vec<usize>,
    /// Adjacent pairs with n = m + 1, stored as (m, n).
    pub near_neighbor_elements: Vec<(usize, usize)>,
    /// All other upper-triangle pairs with n > m + 1, stored as (m, n).
    pub regular_elements: Vec<(usize, usize)>,
}

/// Classify all upper-triangle (m, n) pairs for an N-segment mesh.
pub fn classify(n_segments: usize) -> ElementClassification {
    let mut self_elements = Vec::with_capacity(n_segments);
    let mut near_neighbor_elements = Vec::with_capacity(n_segments.saturating_sub(1));
    let mut regular_elements =
        Vec::with_capacity(n_segments.saturating_mul(n_segments.saturating_sub(1)) / 2);

    for m in 0..n_segments {
        // Self element: m = n
        self_elements.push(m);

        // Near-neighbor: n = m + 1
        if m + 1 < n_segments {
            near_neighbor_elements.push((m, m + 1));
        }

        // Regular: n > m + 1
        for n in (m + 2)..n_segments {
            regular_elements.push((m, n));
        }
    }

    ElementClassification {
        self_elements,
        near_neighbor_elements,
        regular_elements,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n4_counts() {
        // N=4: 4 self + 3 near + 3 regular = 10 upper-triangle entries
        let c = classify(4);
        assert_eq!(c.self_elements.len(), 4);
        assert_eq!(c.near_neighbor_elements.len(), 3);
        assert_eq!(c.regular_elements.len(), 3);
        assert_eq!(
            c.self_elements.len() + c.near_neighbor_elements.len() + c.regular_elements.len(),
            10
        );
    }

    #[test]
    fn n4_self_elements() {
        let c = classify(4);
        assert_eq!(c.self_elements, vec![0, 1, 2, 3]);
    }

    #[test]
    fn n4_near_neighbor_elements() {
        let c = classify(4);
        assert_eq!(
            c.near_neighbor_elements,
            vec![(0, 1), (1, 2), (2, 3)]
        );
    }

    #[test]
    fn n4_regular_elements() {
        let c = classify(4);
        assert_eq!(
            c.regular_elements,
            vec![(0, 2), (0, 3), (1, 3)]
        );
    }

    #[test]
    fn total_equals_upper_triangle() {
        // For any N, total should be N(N+1)/2
        for n in 0..=10 {
            let c = classify(n);
            let total =
                c.self_elements.len() + c.near_neighbor_elements.len() + c.regular_elements.len();
            assert_eq!(total, n * (n + 1) / 2, "N={n}");
        }
    }

    #[test]
    fn n1_single_segment() {
        let c = classify(1);
        assert_eq!(c.self_elements.len(), 1);
        assert_eq!(c.near_neighbor_elements.len(), 0);
        assert_eq!(c.regular_elements.len(), 0);
    }

    #[test]
    fn n0_empty() {
        let c = classify(0);
        assert_eq!(c.self_elements.len(), 0);
        assert_eq!(c.near_neighbor_elements.len(), 0);
        assert_eq!(c.regular_elements.len(), 0);
    }
}
