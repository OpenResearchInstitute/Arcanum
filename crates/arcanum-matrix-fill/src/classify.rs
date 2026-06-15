use std::collections::HashSet;

use arcanum_geometry::mesh::{EndpointSide, Mesh};

/// A near-neighbor segment pair with endpoint-side information.
///
/// The shared endpoint (junction or intra-wire connection) lies at
/// `m_side` of segment `m` and `n_side` of segment `n`.
pub struct NearNeighborPair {
    pub m: usize,
    pub n: usize,
    pub m_side: EndpointSide,
    pub n_side: EndpointSide,
}

/// Result of classifying all upper-triangle element pairs into integration
/// categories.
///
/// Every (m, n) pair with m ≤ n is classified as self, near-neighbor, or
/// regular. Only the upper triangle is computed; the lower triangle is filled
/// by symmetry copy after the parallel fill.
pub struct ElementClassification {
    /// Diagonal indices: m = n. Each entry is a segment index.
    pub self_elements: Vec<usize>,
    /// Adjacent pairs sharing a junction or intra-wire endpoint.
    pub near_neighbor_elements: Vec<NearNeighborPair>,
    /// All other upper-triangle pairs, stored as (m, n) with m < n.
    pub regular_elements: Vec<(usize, usize)>,
}

/// Classify all upper-triangle (m, n) pairs using the mesh's geometric
/// connectivity (junctions and intra-wire adjacency) rather than index order.
pub fn classify(mesh: &Mesh) -> ElementClassification {
    let n_segments = mesh.segments.len();

    let mut self_elements = Vec::with_capacity(n_segments);
    for m in 0..n_segments {
        self_elements.push(m);
    }

    let mut near_set: HashSet<(usize, usize)> = HashSet::new();
    let mut near_neighbor_elements = Vec::new();

    // (a) Intra-wire adjacent: consecutive segments on the same wire.
    //     The end of segment m connects to the start of segment m+1.
    for m in 0..n_segments.saturating_sub(1) {
        let n = m + 1;
        if mesh.segments[m].wire_index == mesh.segments[n].wire_index {
            near_set.insert((m, n));
            near_neighbor_elements.push(NearNeighborPair {
                m,
                n,
                m_side: EndpointSide::End,
                n_side: EndpointSide::Start,
            });
        }
    }

    // (b) Cross-wire junction pairs: segments sharing a junction endpoint.
    for junction in &mesh.junctions {
        let eps = &junction.endpoints;
        for i in 0..eps.len() {
            for j in (i + 1)..eps.len() {
                let (a, b) = (&eps[i], &eps[j]);
                let (m, n, m_side, n_side) = if a.segment_index <= b.segment_index {
                    (a.segment_index, b.segment_index, a.side, b.side)
                } else {
                    (b.segment_index, a.segment_index, b.side, a.side)
                };
                // Skip self-pairs (m == n) and duplicates already found
                // via intra-wire adjacency.
                if m == n || near_set.contains(&(m, n)) {
                    continue;
                }
                near_set.insert((m, n));
                near_neighbor_elements.push(NearNeighborPair {
                    m,
                    n,
                    m_side,
                    n_side,
                });
            }
        }
    }

    // Regular: all upper-triangle pairs not in self or near-neighbor.
    let mut regular_elements =
        Vec::with_capacity(n_segments.saturating_mul(n_segments.saturating_sub(1)) / 2);
    for m in 0..n_segments {
        for n in (m + 1)..n_segments {
            if !near_set.contains(&(m, n)) {
                regular_elements.push((m, n));
            }
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
    use arcanum_geometry::mesh::{
        CurveParams, GroundDescriptor, GroundType, Junction, LinearParams, Material, Segment,
        SegmentEndpoint, TagMap,
    };
    use nalgebra::Vector3;

    fn straight_wire_mesh(n_segments: usize) -> Mesh {
        let segments: Vec<Segment> = (0..n_segments)
            .map(|i| {
                let z0 = i as f64 * 0.05;
                let z1 = z0 + 0.05;
                Segment {
                    curve: CurveParams::Linear(LinearParams {
                        start: Vector3::new(0.0, 0.0, z0),
                        end: Vector3::new(0.0, 0.0, z1),
                    }),
                    wire_radius: 0.001,
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
    fn n4_counts() {
        // N=4 single wire: 4 self + 3 near + 3 regular = 10 upper-triangle entries
        let mesh = straight_wire_mesh(4);
        let c = classify(&mesh);
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
        let mesh = straight_wire_mesh(4);
        let c = classify(&mesh);
        assert_eq!(c.self_elements, vec![0, 1, 2, 3]);
    }

    #[test]
    fn n4_near_neighbor_elements() {
        let mesh = straight_wire_mesh(4);
        let c = classify(&mesh);
        let pairs: Vec<(usize, usize)> = c
            .near_neighbor_elements
            .iter()
            .map(|p| (p.m, p.n))
            .collect();
        assert_eq!(pairs, vec![(0, 1), (1, 2), (2, 3)]);
    }

    #[test]
    fn n4_regular_elements() {
        let mesh = straight_wire_mesh(4);
        let c = classify(&mesh);
        assert_eq!(c.regular_elements, vec![(0, 2), (0, 3), (1, 3)]);
    }

    #[test]
    fn total_equals_upper_triangle() {
        // For any N on a single wire, total should be N(N+1)/2
        for n in 0..=10 {
            let mesh = straight_wire_mesh(n);
            let c = classify(&mesh);
            let total = c.self_elements.len()
                + c.near_neighbor_elements.len()
                + c.regular_elements.len();
            assert_eq!(total, n * (n + 1) / 2, "N={n}");
        }
    }

    #[test]
    fn n1_single_segment() {
        let mesh = straight_wire_mesh(1);
        let c = classify(&mesh);
        assert_eq!(c.self_elements.len(), 1);
        assert_eq!(c.near_neighbor_elements.len(), 0);
        assert_eq!(c.regular_elements.len(), 0);
    }

    #[test]
    fn n0_empty() {
        let mesh = straight_wire_mesh(0);
        let c = classify(&mesh);
        assert_eq!(c.self_elements.len(), 0);
        assert_eq!(c.near_neighbor_elements.len(), 0);
        assert_eq!(c.regular_elements.len(), 0);
    }

    #[test]
    fn two_separate_wires_no_near_neighbors() {
        // Two single-segment wires with no junction: no near-neighbor pairs.
        let segments = vec![
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(0.0, 0.0, 0.0),
                    end: Vector3::new(0.0, 0.0, 0.05),
                }),
                wire_radius: 0.001,
                material: Material::PEC,
                tag: 1,
                segment_index: 0,
                wire_index: 0,
                is_image: false,
            },
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(1.0, 0.0, 0.0),
                    end: Vector3::new(1.0, 0.0, 0.05),
                }),
                wire_radius: 0.001,
                material: Material::PEC,
                tag: 2,
                segment_index: 1,
                wire_index: 1,
                is_image: false,
            },
        ];

        let mesh = Mesh {
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
        };

        let c = classify(&mesh);
        assert_eq!(c.self_elements.len(), 2);
        assert_eq!(c.near_neighbor_elements.len(), 0);
        assert_eq!(c.regular_elements.len(), 1);
        assert_eq!(c.regular_elements[0], (0, 1));
    }

    #[test]
    fn cross_wire_junction_creates_near_neighbor() {
        // Two segments on different wires joined at a junction (T-junction style).
        let segments = vec![
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(0.0, 0.0, 0.0),
                    end: Vector3::new(0.0, 0.0, 0.05),
                }),
                wire_radius: 0.001,
                material: Material::PEC,
                tag: 1,
                segment_index: 0,
                wire_index: 0,
                is_image: false,
            },
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(0.0, 0.0, 0.05),
                    end: Vector3::new(0.05, 0.0, 0.05),
                }),
                wire_radius: 0.001,
                material: Material::PEC,
                tag: 2,
                segment_index: 1,
                wire_index: 1,
                is_image: false,
            },
        ];

        let mesh = Mesh {
            segments,
            junctions: vec![Junction {
                junction_index: 0,
                endpoints: vec![
                    SegmentEndpoint {
                        segment_index: 0,
                        side: EndpointSide::End,
                    },
                    SegmentEndpoint {
                        segment_index: 1,
                        side: EndpointSide::Start,
                    },
                ],
                is_self_loop: false,
            }],
            endpoint_junction: vec![None, Some(0), Some(0), None],
            ground: GroundDescriptor {
                ground_type: GroundType::None,
                conductivity: None,
                permittivity: None,
                images_generated: false,
            },
            tag_map: TagMap::default(),
        };

        let c = classify(&mesh);
        assert_eq!(c.self_elements.len(), 2);
        assert_eq!(c.near_neighbor_elements.len(), 1);
        assert_eq!(c.regular_elements.len(), 0);

        let pair = &c.near_neighbor_elements[0];
        assert_eq!(pair.m, 0);
        assert_eq!(pair.n, 1);
        assert!(matches!(pair.m_side, EndpointSide::End));
        assert!(matches!(pair.n_side, EndpointSide::Start));
    }
}
