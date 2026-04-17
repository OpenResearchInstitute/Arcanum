// tagmap.rs — V-TAG validation cases (Step 8)

use arcanum_nec_import::{
    GeometricGround, GeometryTransforms, MeshInput, StraightWire, WireDescription,
};

use crate::build_mesh;

// ─────────────────────────────────────────────────────────────────────────────
// V-TAG-001 — Multiple wires, tag map correctness
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_tag_001_multiple_wires_tag_map() {
    // GW 1 3 0.0 0.0 -0.25 0.0 0.0 0.25 0.001  → segments 0, 1, 2
    // GW 2 5 0.0 0.0  0.5  1.0 0.0 0.5  0.001  → segments 3, 4, 5, 6, 7
    let wires = vec![
        WireDescription::Straight(StraightWire {
            tag: 1,
            segment_count: 3,
            x1: 0.0, y1: 0.0, z1: -0.25,
            x2: 0.0, y2: 0.0, z2: 0.25,
            radius: 0.001,
        }),
        WireDescription::Straight(StraightWire {
            tag: 2,
            segment_count: 5,
            x1: 0.0, y1: 0.0, z1: 0.5,
            x2: 1.0, y2: 0.0, z2: 0.5,
            radius: 0.001,
        }),
    ];

    let (mesh, _) = build_mesh(
        MeshInput {
            wires,
            ground: GeometricGround::default(),
            gpflag: 0,
            transforms: GeometryTransforms::default(),
        },
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 8, "expected 8 total segments");

    // Tag 1 → segment indices [0, 2].
    let (first1, last1) = mesh.tag_map.get(1).expect("tag 1 not in tag map");
    assert_eq!(first1, 0);
    assert_eq!(last1, 2);
    assert_eq!(mesh.tag_map.segment_count(1), Some(3));

    // Tag 2 → segment indices [3, 7].
    let (first2, last2) = mesh.tag_map.get(2).expect("tag 2 not in tag map");
    assert_eq!(first2, 3);
    assert_eq!(last2, 7);
    assert_eq!(mesh.tag_map.segment_count(2), Some(5));

    // Verify actual segment tags match.
    for k in 0..3usize {
        assert_eq!(mesh.segments[k].tag, 1, "segment {} should have tag 1", k);
    }
    for k in 3..8usize {
        assert_eq!(mesh.segments[k].tag, 2, "segment {} should have tag 2", k);
    }

    // Segment indices are correct.
    for (k, seg) in mesh.segments.iter().enumerate() {
        assert_eq!(seg.segment_index, k, "segment_index mismatch at k={}", k);
    }
}
