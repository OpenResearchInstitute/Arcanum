// junctions.rs — Junction detection (Step 6)
//
// Algorithm:
//   1. Enumerate all 2N segment endpoints.
//   2. For each pair (i, j) compute distance d.
//      ε = min(radius_i, radius_j) × 0.01
//      d < ε        → merge into same junction
//      ε ≤ d < 10ε  → near-coincident warning (no junction)
//   3. Collect merged groups into Junction records.
//   4. Build bidirectional endpoint_junction map.
//
// A junction is flagged is_self_loop when its endpoints all belong to the
// same wire AND the group spans at least one Start-End pair from that wire.

use crate::errors::{GeometryWarning, GeometryWarningKind, GeometryWarnings};
use crate::mesh::{EndpointSide, Junction, Segment, SegmentEndpoint};
use nalgebra::Vector3;

pub(crate) fn detect(
    segments: &[Segment],
    warnings: &mut GeometryWarnings,
) -> (Vec<Junction>, Vec<Option<usize>>) {
    let n = segments.len();
    if n == 0 {
        return (Vec::new(), Vec::new());
    }

    let endpoint_count = 2 * n;

    // Endpoint index encoding: seg_idx * 2 + 0 = Start, seg_idx * 2 + 1 = End.
    let ep_point = |ep_idx: usize| -> Vector3<f64> {
        let seg_idx = ep_idx / 2;
        if ep_idx % 2 == 0 {
            segments[seg_idx].start()
        } else {
            segments[seg_idx].end()
        }
    };
    let ep_radius = |ep_idx: usize| -> f64 { segments[ep_idx / 2].wire_radius };

    // Union-find over endpoints.
    let mut parent: Vec<usize> = (0..endpoint_count).collect();
    let find = |parent: &mut Vec<usize>, mut x: usize| -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]]; // path compression
            x = parent[x];
        }
        x
    };

    // Scan all pairs, merge coincident endpoints, warn on near-coincident.
    for i in 0..endpoint_count {
        let pi = ep_point(i);
        let ri = ep_radius(i);
        for j in (i + 1)..endpoint_count {
            // Skip intra-wire adjacent pairs: within the same wire, consecutive
            // segment end→start connections are implicit and never junctions.
            let si = i / 2;
            let sj = j / 2;
            if segments[si].wire_index == segments[sj].wire_index {
                // Check adjacency: (si End, sj Start) with sj == si + 1.
                let i_is_end   = i % 2 == 1;
                let j_is_start = j % 2 == 0;
                if i_is_end && j_is_start && sj == si + 1 {
                    continue;
                }
                // Symmetric: (sj End, si Start) with si == sj + 1 (won't occur
                // since i < j, but kept for clarity).
            }

            let pj = ep_point(j);
            let rj = ep_radius(j);
            let eps = ri.min(rj) * 0.01;
            let dist = (pi - pj).norm();

            if dist < eps {
                // Merge i and j.
                let ri_root = find(&mut parent, i);
                let rj_root = find(&mut parent, j);
                if ri_root != rj_root {
                    parent[rj_root] = ri_root;
                }
            } else if dist < 10.0 * eps {
                // Near-coincident — emit warning but do NOT merge.
                let si = i / 2;
                let sj = j / 2;
                let side_i = if i % 2 == 0 { "start" } else { "end" };
                let side_j = if j % 2 == 0 { "start" } else { "end" };
                warnings.push(GeometryWarning::new(
                    GeometryWarningKind::NearCoincidentEndpoints,
                    format!(
                        "near-coincident endpoints: seg {} {} ({:.4},{:.4},{:.4}) and \
                         seg {} {} ({:.4},{:.4},{:.4}), gap = {:.3e} m",
                        si,
                        side_i,
                        pi.x,
                        pi.y,
                        pi.z,
                        sj,
                        side_j,
                        pj.x,
                        pj.y,
                        pj.z,
                        dist
                    ),
                ));
            }
        }
    }

    // Canonicalize parents (flatten path compression).
    for i in 0..endpoint_count {
        let root = find(&mut parent, i);
        parent[i] = root;
    }

    // Group endpoints by root.
    // Only form a Junction if the group has more than one endpoint.
    let mut groups: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..endpoint_count {
        groups.entry(parent[i]).or_default().push(i);
    }

    // Build junctions and endpoint_junction.
    let mut junctions: Vec<Junction> = Vec::new();
    let mut endpoint_junction: Vec<Option<usize>> = vec![None; endpoint_count];

    for (_root, members) in &groups {
        if members.len() < 2 {
            // Single endpoint — free end, not a junction.
            continue;
        }

        let junction_index = junctions.len();

        // Build endpoint records.
        let endpoints: Vec<SegmentEndpoint> = members
            .iter()
            .map(|&ep_idx| SegmentEndpoint {
                segment_index: ep_idx / 2,
                side: if ep_idx % 2 == 0 {
                    EndpointSide::Start
                } else {
                    EndpointSide::End
                },
            })
            .collect();

        // Self-loop detection: all endpoints belong to the same wire, and the
        // group includes at least one Start and one End from that wire.
        let wire_indices: std::collections::HashSet<usize> = members
            .iter()
            .map(|&ep_idx| segments[ep_idx / 2].wire_index)
            .collect();
        let sides: std::collections::HashSet<u8> = members
            .iter()
            .map(|&ep_idx| (ep_idx % 2) as u8)
            .collect();
        let is_self_loop = wire_indices.len() == 1 && sides.len() == 2;

        junctions.push(Junction {
            junction_index,
            endpoints,
            is_self_loop,
        });

        for &ep_idx in members {
            endpoint_junction[ep_idx] = Some(junction_index);
        }
    }

    (junctions, endpoint_junction)
}
