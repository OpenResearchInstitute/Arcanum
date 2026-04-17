// arc.rs — V-ARC validation cases (Steps 3 and 6)
//
// V-ARC-001 and V-ARC-002 test arc discretization endpoints (Step 3).
// V-ARC-002 self-loop junction assertion is TODO (Step 6).
// V-ARC-003 near-coincident warning assertion is TODO (Step 10).

use std::f64::consts::PI;

use arcanum_nec_import::{
    ArcWire, GeometricGround, GeometryTransforms, MeshInput, WireDescription,
};

use crate::build_mesh;
use crate::mesh::CurveParams;

use super::approx_eq;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn ga(
    tag: u32,
    n: u32,
    radius: f64,
    angle1: f64,
    angle2: f64,
    wire_radius: f64,
) -> WireDescription {
    WireDescription::Arc(ArcWire {
        tag,
        segment_count: n,
        arc_radius: radius,
        angle1,
        angle2,
        radius: wire_radius,
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
// V-ARC-001 — Semicircular arc, 4 segments
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_arc_001_semicircle_4_segments() {
    // GA 1 4 0.5 0.0 180.0 0.001
    // Arc in XZ plane: r(θ) = (R cosθ, 0, R sinθ), θ from 0° to 180°.
    // Each segment subtends 45°.
    let (mesh, _warnings) =
        build_mesh(free_space(vec![ga(1, 4, 0.5, 0.0, 180.0, 0.001)]), None)
            .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 4);

    let tol = 1e-4;

    // Expected endpoints at θ = 0°, 45°, 90°, 135°, 180°.
    let r = 0.5_f64;
    let angles_deg = [0.0_f64, 45.0, 90.0, 135.0, 180.0];
    let pts: Vec<(f64, f64)> = angles_deg
        .iter()
        .map(|&deg| {
            let theta = deg.to_radians();
            (r * theta.cos(), r * theta.sin())
        })
        .collect();

    // Segment 0 start: θ = 0° → (0.5, 0, 0)
    approx_eq!(mesh.segments[0].start().x, pts[0].0, tol);
    approx_eq!(mesh.segments[0].start().y, 0.0, tol);
    approx_eq!(mesh.segments[0].start().z, pts[0].1, tol);

    // Segment 0 end / Segment 1 start: θ = 45° → (0.3536, 0, 0.3536)
    approx_eq!(mesh.segments[0].end().x, pts[1].0, tol);
    approx_eq!(mesh.segments[0].end().z, pts[1].1, tol);
    approx_eq!(mesh.segments[1].start().x, pts[1].0, tol);
    approx_eq!(mesh.segments[1].start().z, pts[1].1, tol);

    // Segment 1 end: θ = 90° → (0, 0, 0.5)
    approx_eq!(mesh.segments[1].end().x, pts[2].0, tol);
    approx_eq!(mesh.segments[1].end().z, pts[2].1, tol);

    // Segment 2 end: θ = 135° → (-0.3536, 0, 0.3536)
    approx_eq!(mesh.segments[2].end().x, pts[3].0, tol);
    approx_eq!(mesh.segments[2].end().z, pts[3].1, tol);

    // Segment 3 end: θ = 180° → (-0.5, 0, 0)
    approx_eq!(mesh.segments[3].end().x, pts[4].0, tol);
    approx_eq!(mesh.segments[3].end().z, pts[4].1, tol);

    // y = 0 for all endpoints (arc is in XZ plane).
    for seg in &mesh.segments {
        approx_eq!(seg.start().y, 0.0, tol);
        approx_eq!(seg.end().y, 0.0, tol);
    }

    // CurveParams carries correct radius and monotone angles within [0, π].
    for (k, seg) in mesh.segments.iter().enumerate() {
        if let CurveParams::Arc(p) = &seg.curve {
            approx_eq!(p.radius, 0.5);
            assert!(
                p.theta1 >= 0.0 && p.theta1 < PI + 1e-9,
                "theta1 out of range: {}",
                p.theta1
            );
            assert!(p.theta2 > p.theta1, "theta2 <= theta1 for segment {}", k);
            assert!(p.theta2 <= PI + 1e-9, "theta2 out of range: {}", p.theta2);
        } else {
            panic!("expected Arc curve for segment {}", k);
        }
    }

    // Wire radius and tag.
    for seg in &mesh.segments {
        approx_eq!(seg.wire_radius, 0.001);
        assert_eq!(seg.tag, 1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ARC-002 — Full circle (loop antenna), 8 segments
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_arc_002_full_circle_8_segments() {
    // GA 1 8 0.25 0.0 360.0 0.001
    let (mesh, _warnings) =
        build_mesh(free_space(vec![ga(1, 8, 0.25, 0.0, 360.0, 0.001)]), None)
            .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 8);

    let tol = 1e-4;

    // Adjacent segment endpoints must be coincident (geometric continuity).
    for k in 0..7usize {
        let end_k = mesh.segments[k].end();
        let start_k1 = mesh.segments[k + 1].start();
        let gap = (end_k - start_k1).norm();
        assert!(
            gap < 1e-12,
            "gap between seg {} end and seg {} start: {} m",
            k,
            k + 1,
            gap
        );
    }

    // Closure: seg 7 end ≈ seg 0 start (the loop closes).
    let first_start = mesh.segments[0].start();
    let last_end = mesh.segments[7].end();
    approx_eq!(first_start.x, last_end.x, tol);
    approx_eq!(first_start.y, last_end.y, tol);
    approx_eq!(first_start.z, last_end.z, tol);

    // Seg 0 start at θ = 0°: (0.25, 0, 0).
    approx_eq!(first_start.x, 0.25, tol);
    approx_eq!(first_start.y, 0.0, tol);
    approx_eq!(first_start.z, 0.0, tol);

    // All y = 0 (arc in XZ plane).
    for seg in &mesh.segments {
        approx_eq!(seg.start().y, 0.0, tol);
        approx_eq!(seg.end().y, 0.0, tol);
    }

    // Self-loop junction: seg 7 end connects to seg 0 start; is_self_loop = true.
    assert_eq!(mesh.junctions.len(), 1, "expected 1 (self-loop) junction");
    let j = &mesh.junctions[0];
    assert!(j.is_self_loop, "junction should be flagged as self-loop");
    assert_eq!(j.endpoints.len(), 2);

    use crate::mesh::EndpointSide;
    let has_seg0_start = j.endpoints.iter().any(|ep| ep.segment_index == 0 && ep.side == EndpointSide::Start);
    let has_seg7_end   = j.endpoints.iter().any(|ep| ep.segment_index == 7 && ep.side == EndpointSide::End);
    assert!(has_seg0_start, "self-loop junction missing seg 0 Start");
    assert!(has_seg7_end,   "self-loop junction missing seg 7 End");
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ARC-003 — Near-coincident endpoints (359° arc)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_arc_003_near_coincident_gap() {
    // GA 1 8 0.25 0.0 359.0 0.001
    // Arc almost closes — gap between start and end is very small.
    let (mesh, _warnings) =
        build_mesh(free_space(vec![ga(1, 8, 0.25, 0.0, 359.0, 0.001)]), None)
            .expect("build_mesh must succeed (no hard error)");

    assert_eq!(mesh.segments.len(), 8, "expected 8 segments");

    let tol = 1e-4;

    // Seg 0 start: θ = 0° → (0.25, 0, 0).
    approx_eq!(mesh.segments[0].start().x, 0.25, tol);
    approx_eq!(mesh.segments[0].start().z, 0.0, tol);

    // Seg 7 end: θ = 359° → very close to (0.25, 0, 0) but not equal.
    let theta_end = 359.0_f64.to_radians();
    let expected_end_x = 0.25 * theta_end.cos();
    let expected_end_z = 0.25 * theta_end.sin();
    approx_eq!(mesh.segments[7].end().x, expected_end_x, tol);
    approx_eq!(mesh.segments[7].end().z, expected_end_z, tol);

    // The gap is nonzero — start and end are NOT coincident.
    let start = mesh.segments[0].start();
    let end = mesh.segments[7].end();
    let gap = (start - end).norm();
    assert!(gap > 0.0, "expected nonzero gap for 359° arc");
    assert!(gap < 0.01, "gap unexpectedly large: {} m", gap);

    // No junction created at the gap — endpoints are near-coincident but not merged.
    assert!(mesh.junctions.is_empty(), "expected no junctions for 359° arc");
    // TODO(Step 10): assert a NearCoincidentEndpoints warning was emitted.
}
