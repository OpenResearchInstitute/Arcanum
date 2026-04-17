// linear.rs — V-LIN validation cases (Steps 2 and 6)
//
// V-LIN-001, V-LIN-002, V-LIN-005, V-LIN-006 test discretization (Step 2).
// V-LIN-003, V-LIN-004 segment assertions are in Step 2; junction assertions
// are marked TODO and will be filled in when Step 6 (junctions.rs) is complete.

use arcanum_nec_import::{
    GeometricGround, GeometryTransforms, MeshInput, StraightWire, WireDescription,
};

use crate::errors::GeometryErrorKind;
use crate::mesh::CurveParams;
use crate::{build_mesh, mesh::Material};

use super::approx_eq;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn gw(
    tag: u32,
    n: u32,
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    radius: f64,
) -> WireDescription {
    WireDescription::Straight(StraightWire {
        tag,
        segment_count: n,
        x1,
        y1,
        z1,
        x2,
        y2,
        z2,
        radius,
    })
}

fn free_space(wires: Vec<WireDescription>) -> MeshInput {
    MeshInput {
        wires,
        ground: GeometricGround::default(),
        gpflag: 0,
        transforms: GeometryTransforms::default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-001 — Single straight wire, two segments, z-axis
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_001_two_segment_dipole() {
    // GW 1 2 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
    let (mesh, _warnings) =
        build_mesh(free_space(vec![gw(1, 2, 0.0, 0.0, -0.25, 0.0, 0.0, 0.25, 0.001)]), None)
            .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 2, "expected 2 segments");

    let s0 = &mesh.segments[0];
    let s1 = &mesh.segments[1];

    // Segment 0: (0, 0, -0.25) → (0, 0, 0.0)
    approx_eq!(s0.start().x, 0.0);
    approx_eq!(s0.start().y, 0.0);
    approx_eq!(s0.start().z, -0.25);
    approx_eq!(s0.end().x, 0.0);
    approx_eq!(s0.end().y, 0.0);
    approx_eq!(s0.end().z, 0.0);

    // Segment 1: (0, 0, 0.0) → (0, 0, 0.25)
    approx_eq!(s1.start().x, 0.0);
    approx_eq!(s1.start().y, 0.0);
    approx_eq!(s1.start().z, 0.0);
    approx_eq!(s1.end().x, 0.0);
    approx_eq!(s1.end().y, 0.0);
    approx_eq!(s1.end().z, 0.25);

    // Adjacent endpoints are coincident (shared midpoint).
    approx_eq!(s0.end().z, s1.start().z);

    // Radius, material, tag.
    approx_eq!(s0.wire_radius, 0.001);
    approx_eq!(s1.wire_radius, 0.001);
    assert_eq!(s0.material, Material::PEC);
    assert_eq!(s1.material, Material::PEC);
    assert_eq!(s0.tag, 1);
    assert_eq!(s1.tag, 1);

    // Lengths via CurveParams.
    if let CurveParams::Linear(p) = &s0.curve {
        approx_eq!(p.length(), 0.25);
    } else {
        panic!("expected Linear curve");
    }
    if let CurveParams::Linear(p) = &s1.curve {
        approx_eq!(p.length(), 0.25);
    } else {
        panic!("expected Linear curve");
    }

    // Ground descriptor: free space.
    assert_eq!(mesh.ground.ground_type, crate::mesh::GroundType::None);

    // Intra-wire adjacent connections are NOT junctions (the midpoint at z=0 is
    // an implicit within-wire segment boundary, not a cross-wire junction).
    // Both outer endpoints (z=-0.25 and z=0.25) are free.
    use crate::mesh::EndpointSide;
    assert!(mesh.junctions.is_empty(), "intra-wire midpoint should not be a junction");
    assert!(mesh.junction_at(0, &EndpointSide::Start).is_none(), "seg 0 start should be free");
    assert!(mesh.junction_at(1, &EndpointSide::End).is_none(),   "seg 1 end should be free");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-002 — Single segment, x-axis
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_002_single_segment_x_axis() {
    // GW 1 1 0.0 0.0 0.0 1.0 0.0 0.0 0.005
    let (mesh, _warnings) =
        build_mesh(free_space(vec![gw(1, 1, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.005)]), None)
            .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 1, "expected 1 segment");

    let s0 = &mesh.segments[0];

    approx_eq!(s0.start().x, 0.0);
    approx_eq!(s0.start().y, 0.0);
    approx_eq!(s0.start().z, 0.0);
    approx_eq!(s0.end().x, 1.0);
    approx_eq!(s0.end().y, 0.0);
    approx_eq!(s0.end().z, 0.0);

    approx_eq!(s0.wire_radius, 0.005);
    assert_eq!(s0.tag, 1);
    assert_eq!(s0.segment_index, 0);

    if let CurveParams::Linear(p) = &s0.curve {
        approx_eq!(p.length(), 1.0);
    } else {
        panic!("expected Linear curve");
    }

    // Tag map: tag 1 → segment indices [0, 0].
    let range = mesh.tag_map.get(1).expect("tag 1 not in tag map");
    assert_eq!(range, (0, 0));
    assert_eq!(mesh.tag_map.segment_count(1), Some(1));

    // No junctions (single segment, two free endpoints).
    assert!(mesh.junctions.is_empty(), "expected no junctions for single segment");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-003 — Two wires joined at a junction
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_003_two_wires_shared_endpoint() {
    // GW 1 3 0.0 0.0 0.0 1.0 0.0 0.0 0.001
    // GW 2 3 1.0 0.0 0.0 2.0 0.0 0.0 0.001
    let (mesh, _warnings) = build_mesh(
        free_space(vec![
            gw(1, 3, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.001),
            gw(2, 3, 1.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.001),
        ]),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 6, "expected 6 segments");

    // Wire 1 segments: indices 0, 1, 2 — evenly spaced along x from 0 to 1.
    let step = 1.0 / 3.0;
    for k in 0..3usize {
        let s = &mesh.segments[k];
        approx_eq!(s.start().x, k as f64 * step);
        approx_eq!(s.end().x, (k + 1) as f64 * step);
        approx_eq!(s.start().y, 0.0);
        approx_eq!(s.start().z, 0.0);
        assert_eq!(s.tag, 1);
        assert_eq!(s.wire_index, 0);
    }

    // Wire 2 segments: indices 3, 4, 5 — evenly spaced along x from 1 to 2.
    for k in 0..3usize {
        let s = &mesh.segments[3 + k];
        approx_eq!(s.start().x, 1.0 + k as f64 * step);
        approx_eq!(s.end().x, 1.0 + (k + 1) as f64 * step);
        assert_eq!(s.tag, 2);
        assert_eq!(s.wire_index, 1);
    }

    // Shared endpoint at (1, 0, 0): wire 1 seg 2 end == wire 2 seg 3 start.
    approx_eq!(mesh.segments[2].end().x, 1.0);
    approx_eq!(mesh.segments[3].start().x, 1.0);

    // Tag map.
    assert_eq!(mesh.tag_map.get(1), Some((0, 2)));
    assert_eq!(mesh.tag_map.get(2), Some((3, 5)));

    // One junction at (1,0,0) connecting seg 2 end to seg 3 start.
    assert_eq!(mesh.junctions.len(), 1, "expected 1 junction");
    let j = &mesh.junctions[0];
    assert_eq!(j.endpoints.len(), 2, "junction valence should be 2");
    assert!(!j.is_self_loop);

    use crate::mesh::EndpointSide;
    let has_seg2_end   = j.endpoints.iter().any(|ep| ep.segment_index == 2 && ep.side == EndpointSide::End);
    let has_seg3_start = j.endpoints.iter().any(|ep| ep.segment_index == 3 && ep.side == EndpointSide::Start);
    assert!(has_seg2_end,   "junction missing seg 2 End");
    assert!(has_seg3_start, "junction missing seg 3 Start");

    // Free endpoints at (0,0,0) and (2,0,0).
    assert!(mesh.junction_at(0, &EndpointSide::Start).is_none(), "seg 0 start should be free");
    assert!(mesh.junction_at(5, &EndpointSide::End).is_none(),   "seg 5 end should be free");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-004 — T-junction, three wires meeting at origin
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_004_t_junction_three_wires() {
    // GW 1 2 -0.5 0.0 0.0  0.5 0.0 0.0 0.001
    // GW 2 2  0.0 0.0 0.0  0.0 0.5 0.0 0.001
    // GW 3 2  0.0 0.0 0.0  0.0 0.0 0.5 0.001
    let (mesh, _warnings) = build_mesh(
        free_space(vec![
            gw(1, 2, -0.5, 0.0, 0.0, 0.5, 0.0, 0.0, 0.001),
            gw(2, 2, 0.0, 0.0, 0.0, 0.0, 0.5, 0.0, 0.001),
            gw(3, 2, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.001),
        ]),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 6, "expected 6 segments");

    // Wire 1: midpoint at origin (seg 0 end, seg 1 start).
    approx_eq!(mesh.segments[0].end().x, 0.0);
    approx_eq!(mesh.segments[0].end().y, 0.0);
    approx_eq!(mesh.segments[0].end().z, 0.0);
    approx_eq!(mesh.segments[1].start().x, 0.0);

    // Wire 2: start at origin.
    approx_eq!(mesh.segments[2].start().x, 0.0);
    approx_eq!(mesh.segments[2].start().y, 0.0);
    approx_eq!(mesh.segments[2].start().z, 0.0);

    // Wire 3: start at origin.
    approx_eq!(mesh.segments[4].start().x, 0.0);
    approx_eq!(mesh.segments[4].start().y, 0.0);
    approx_eq!(mesh.segments[4].start().z, 0.0);

    // Wire 1 outer endpoints.
    approx_eq!(mesh.segments[0].start().x, -0.5);
    approx_eq!(mesh.segments[1].end().x, 0.5);

    // Tag map.
    assert_eq!(mesh.tag_map.get(1), Some((0, 1)));
    assert_eq!(mesh.tag_map.get(2), Some((2, 3)));
    assert_eq!(mesh.tag_map.get(3), Some((4, 5)));

    // One junction at (0,0,0) where 3 wires meet.
    // Endpoints in the junction:
    //   seg 0 End and seg 1 Start (wire 1 midpoint, 2 endpoints)
    //   seg 2 Start (wire 2 start)
    //   seg 4 Start (wire 3 start)
    // Total: 4 segment endpoints, representing 3 wires (valence 3).
    assert_eq!(mesh.junctions.len(), 1, "expected 1 junction");
    let j = &mesh.junctions[0];
    assert_eq!(j.endpoints.len(), 4, "T-junction should have 4 segment endpoints (3 wires)");
    assert!(!j.is_self_loop);

    use crate::mesh::EndpointSide;
    let has_seg0_end   = j.endpoints.iter().any(|ep| ep.segment_index == 0 && ep.side == EndpointSide::End);
    let has_seg1_start = j.endpoints.iter().any(|ep| ep.segment_index == 1 && ep.side == EndpointSide::Start);
    let has_seg2_start = j.endpoints.iter().any(|ep| ep.segment_index == 2 && ep.side == EndpointSide::Start);
    let has_seg4_start = j.endpoints.iter().any(|ep| ep.segment_index == 4 && ep.side == EndpointSide::Start);
    assert!(has_seg0_end,   "junction missing seg 0 End (wire 1 midpoint)");
    assert!(has_seg1_start, "junction missing seg 1 Start (wire 1 midpoint)");
    assert!(has_seg2_start, "junction missing seg 2 Start (wire 2)");
    assert!(has_seg4_start, "junction missing seg 4 Start (wire 3)");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-005 — Zero-length wire (hard error)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_005_zero_length_wire_error() {
    // GW 1 4 0.5 0.5 0.5 0.5 0.5 0.5 0.001  — start == end
    let result =
        build_mesh(free_space(vec![gw(1, 4, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.001)]), None);

    let err = result.expect_err("expected hard error for zero-length wire");
    assert_eq!(
        err.kind,
        GeometryErrorKind::ZeroLengthWire,
        "wrong error kind: {:?}",
        err.kind
    );
    assert_eq!(err.wire_index, 0, "wrong wire index in error");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-LIN-006 — Zero segment count (hard error)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_lin_006_zero_segment_count_error() {
    // GW 1 0 0.0 0.0 0.0 1.0 0.0 0.0 0.001  — SEGS = 0
    let result =
        build_mesh(free_space(vec![gw(1, 0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.001)]), None);

    let err = result.expect_err("expected hard error for zero segment count");
    assert_eq!(
        err.kind,
        GeometryErrorKind::ZeroSegmentCount,
        "wrong error kind: {:?}",
        err.kind
    );
    assert_eq!(err.wire_index, 0, "wrong wire index in error");
}
