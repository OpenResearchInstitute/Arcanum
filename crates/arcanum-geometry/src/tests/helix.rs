// helix.rs — V-HEL validation cases (Steps 4 and 7)
//
// V-HEL-001 and V-HEL-002 test helix discretization (Step 4).
// V-HEL-003 PEC image assertions are TODO (Step 7).
//
// NOTE: The GH card strings in validation.md V-HEL-001/002 have an extra
// field and an inconsistent total_length value. Tests here construct
// HelixWire structs directly with the physically correct values that match
// the stated expected outputs (one and five turns respectively).

use arcanum_nec_import::{
    GeometricGround, GeometricGround as NecGeometricGround, GeometryTransforms, GroundType as NecGroundType,
    HelixWire, MeshInput, WireDescription,
};

use crate::build_mesh;
use crate::mesh::CurveParams;

use super::approx_eq;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn helix_wire(
    tag: u32,
    n: u32,
    pitch: f64,
    total_length: f64,
    radius_start: f64,
    radius_end: f64,
    wire_radius: f64,
) -> WireDescription {
    WireDescription::Helix(HelixWire {
        tag,
        segment_count: n,
        pitch,
        total_length,
        radius_start,
        radius_end,
        radius: wire_radius,
        n_turns: total_length / pitch,
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

fn pec_ground(wires: Vec<WireDescription>) -> MeshInput {
    MeshInput {
        wires,
        ground: NecGeometricGround { ground_type: NecGroundType::PEC },
        gpflag: 1,
        transforms: GeometryTransforms::default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-HEL-001 — Single-turn helix, 8 segments
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_hel_001_single_turn_8_segments() {
    // GH 1 8 <pitch=0.0628> <total_length=0.0628> 0.05 0.05 0.001
    // n_turns = total_length / pitch = 1.0
    let pitch = 0.0628_f64;
    let (mesh, _warnings) = build_mesh(
        free_space(vec![helix_wire(1, 8, pitch, pitch, 0.05, 0.05, 0.001)]),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 8);

    let tol = 1e-4;

    // Segment 0 start: t = 0 → (A1, 0, 0) = (0.05, 0.0, 0.0)
    approx_eq!(mesh.segments[0].start().x, 0.05, tol);
    approx_eq!(mesh.segments[0].start().y, 0.0, tol);
    approx_eq!(mesh.segments[0].start().z, 0.0, tol);

    // After one full turn (t = 1): same x,y as start, z = total_length = 0.0628.
    // angle = 2π * n_turns * t = 2π → cos(2π) = 1, sin(2π) = 0.
    approx_eq!(mesh.segments[7].end().x, 0.05, tol);
    approx_eq!(mesh.segments[7].end().y, 0.0, tol);
    approx_eq!(mesh.segments[7].end().z, pitch, tol);

    // Adjacent segment endpoints are coincident (no gaps).
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

    // Each segment subtends 45° = π/4 radians of rotation.
    // Confirm intermediate endpoints at 45° increments.
    use std::f64::consts::PI;
    for k in 0..8usize {
        if let CurveParams::Helix(p) = &mesh.segments[k].curve {
            assert_eq!(p.n_segments, 8);
            assert_eq!(p.segment_index as usize, k);
            approx_eq!(p.total_length, pitch);
            approx_eq!(p.n_turns, 1.0);
            approx_eq!(p.radius_start, 0.05);
            approx_eq!(p.radius_end, 0.05);
        } else {
            panic!("expected Helix curve for segment {}", k);
        }

        // z-coordinate at segment start should be k/8 * total_length.
        let expected_z_start = k as f64 / 8.0 * pitch;
        approx_eq!(mesh.segments[k].start().z, expected_z_start, tol);

        // x,y at segment start: angle = 2π * (k/8).
        let theta = 2.0 * PI * (k as f64 / 8.0);
        approx_eq!(mesh.segments[k].start().x, 0.05 * theta.cos(), tol);
        approx_eq!(mesh.segments[k].start().y, 0.05 * theta.sin(), tol);
    }

    // Wire radius and tag.
    for seg in &mesh.segments {
        approx_eq!(seg.wire_radius, 0.001);
        assert_eq!(seg.tag, 1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-HEL-002 — Multi-turn helix, endpoint continuity
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_hel_002_five_turn_endpoint_continuity() {
    // 5-turn helix: 40 segments at 8 per turn.
    // total_length = 5 * 0.0628 = 0.314 m
    let pitch = 0.0628_f64;
    let total_length = 5.0 * pitch;
    let (mesh, _warnings) = build_mesh(
        free_space(vec![helix_wire(1, 40, pitch, total_length, 0.05, 0.05, 0.001)]),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 40);

    // Primary: geometric continuity — gap between consecutive segment endpoints
    // must be zero to double precision (< 1e-12 m).
    for k in 0..39usize {
        let end_k = mesh.segments[k].end();
        let start_k1 = mesh.segments[k + 1].start();
        let gap = (end_k - start_k1).norm();
        assert!(
            gap < 1e-12,
            "continuity failure: gap between seg {} end and seg {} start = {} m",
            k,
            k + 1,
            gap
        );
    }

    // Final z-coordinate: total_length = 0.314 m.
    let tol = 1e-9;
    approx_eq!(mesh.segments[39].end().z, total_length, tol);

    // Segment 0 start: (0.05, 0.0, 0.0).
    approx_eq!(mesh.segments[0].start().x, 0.05, tol);
    approx_eq!(mesh.segments[0].start().y, 0.0, tol);
    approx_eq!(mesh.segments[0].start().z, 0.0, tol);

    // After 5 full turns (t = 1): angle = 2π * 5 * 1 = 10π → cos = 1, sin = 0.
    // Final endpoint should be at (0.05, 0.0, 0.314).
    approx_eq!(mesh.segments[39].end().x, 0.05, 1e-9);
    approx_eq!(mesh.segments[39].end().y, 0.0, 1e-9);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-HEL-003 — Helix over PEC ground plane
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_hel_003_helix_over_pec_ground() {
    // GH 1 16 0.0628 0.1256 0.05 0.05 0.001  (2-turn helix)
    // GN 1 / GE 1
    let pitch = 0.0628_f64;
    let total_length = 2.0 * pitch;
    let (mesh, _warnings) = build_mesh(
        pec_ground(vec![helix_wire(1, 16, pitch, total_length, 0.05, 0.05, 0.001)]),
        None,
    )
    .expect("build_mesh failed");

    // 16 real + 16 image = 32 total.
    let real_count = mesh.real_segment_count();
    let image_count = mesh.image_segment_count();
    assert_eq!(real_count, 16, "expected 16 real segments");
    assert_eq!(image_count, 16, "expected 16 image segments");
    assert_eq!(mesh.segments.len(), 32);

    // Ground descriptor.
    assert_eq!(mesh.ground.ground_type, crate::mesh::GroundType::PEC);
    assert!(mesh.ground.images_generated);

    let tol = 1e-12;
    // Image segments have z-coordinates negated.
    for k in 0..16usize {
        let real = &mesh.segments[k];
        let image = &mesh.segments[16 + k];
        assert!(image.is_image);
        approx_eq!(image.start().z, -real.start().z, tol);
        approx_eq!(image.end().z, -real.end().z, tol);
        approx_eq!(image.start().x, real.start().x, tol);
        approx_eq!(image.start().y, real.start().y, tol);
    }

    // One junction at z = 0: real segment 0 Start and image segment 16 Start share
    // the helix feed point at the ground plane.
    assert_eq!(mesh.junctions.len(), 1);
    assert!(!mesh.junctions[0].is_self_loop);
}
