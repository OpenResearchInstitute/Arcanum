// curve_eval.rs — Curve evaluation method tests
//
// Tests for CurveParams::evaluate(), tangent(), speed(), arc_length().
// These methods are the prerequisite for Phase 2 quadrature integration.

use std::f64::consts::PI;

use arcanum_nec_import::{
    ArcWire, GeometricGround, GeometryTransforms, GmOperation, HelixWire, MeshInput, StraightWire,
    WireDescription,
};

use crate::build_mesh;

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

fn gh(
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

fn with_gm(wires: Vec<WireDescription>, gm: GmOperation) -> MeshInput {
    MeshInput {
        wires,
        ground: GeometricGround::default(),
        gpflag: 0,
        transforms: GeometryTransforms {
            gs_scale: None,
            gm_ops: vec![gm],
        },
    }
}

fn vec3_approx_eq(a: nalgebra::Vector3<f64>, b: nalgebra::Vector3<f64>, tol: f64) {
    approx_eq!(a.x, b.x, tol);
    approx_eq!(a.y, b.y, tol);
    approx_eq!(a.z, b.z, tol);
}

// ─────────────────────────────────────────────────────────────────────────────
// Linear: evaluate(0) == start, evaluate(1) == end
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn linear_evaluate_endpoints() {
    let (mesh, _) = build_mesh(
        free_space(vec![gw(1, 3, 0.0, 0.0, -0.25, 0.0, 0.0, 0.25, 0.001)]),
        None,
    )
    .unwrap();

    let tol = 1e-12;
    for seg in &mesh.segments {
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }
}

#[test]
fn linear_evaluate_midpoint() {
    let (mesh, _) = build_mesh(
        free_space(vec![gw(1, 1, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 0.001)]),
        None,
    )
    .unwrap();

    let mid = mesh.segments[0].curve.evaluate(0.5);
    let tol = 1e-12;
    approx_eq!(mid.x, 0.5, tol);
    approx_eq!(mid.y, 1.0, tol);
    approx_eq!(mid.z, 1.5, tol);
}

#[test]
fn linear_speed_equals_length() {
    let (mesh, _) = build_mesh(
        free_space(vec![gw(1, 1, 0.0, 0.0, 0.0, 3.0, 4.0, 0.0, 0.001)]),
        None,
    )
    .unwrap();

    // Length = 5.0 (3-4-5 triangle).
    // For linear, speed = |end - start| = segment length, constant over σ.
    let tol = 1e-12;
    approx_eq!(mesh.segments[0].curve.speed(0.0), 5.0, tol);
    approx_eq!(mesh.segments[0].curve.speed(0.5), 5.0, tol);
    approx_eq!(mesh.segments[0].curve.speed(1.0), 5.0, tol);
}

#[test]
fn linear_arc_length() {
    let (mesh, _) = build_mesh(
        free_space(vec![gw(1, 1, 0.0, 0.0, 0.0, 3.0, 4.0, 0.0, 0.001)]),
        None,
    )
    .unwrap();

    approx_eq!(mesh.segments[0].curve.arc_length(), 5.0, 1e-12);
}

#[test]
fn linear_tangent_is_constant() {
    let (mesh, _) = build_mesh(
        free_space(vec![gw(1, 1, 1.0, 2.0, 3.0, 4.0, 6.0, 3.0, 0.001)]),
        None,
    )
    .unwrap();

    let t0 = mesh.segments[0].curve.tangent(0.0);
    let t1 = mesh.segments[0].curve.tangent(0.5);
    let t2 = mesh.segments[0].curve.tangent(1.0);
    let tol = 1e-12;
    vec3_approx_eq(t0, t1, tol);
    vec3_approx_eq(t1, t2, tol);
    // Tangent = end - start = (3, 4, 0)
    approx_eq!(t0.x, 3.0, tol);
    approx_eq!(t0.y, 4.0, tol);
    approx_eq!(t0.z, 0.0, tol);
}

// ─────────────────────────────────────────────────────────────────────────────
// Arc: evaluate(0) == start, evaluate(1) == end
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn arc_evaluate_endpoints() {
    // Semicircular arc, R=0.5, 0° to 180°, 4 segments.
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 4, 0.5, 0.0, 180.0, 0.001)]), None).unwrap();

    let tol = 1e-10;
    for seg in &mesh.segments {
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }
}

#[test]
fn arc_evaluate_midpoint() {
    // Single-segment quarter arc: 0° to 90°, R=1.0
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 1, 1.0, 0.0, 90.0, 0.001)]), None).unwrap();

    // Midpoint at θ = 45°: (cos45°, 0, sin45°) = (√2/2, 0, √2/2)
    let mid = mesh.segments[0].curve.evaluate(0.5);
    let tol = 1e-10;
    let s2 = std::f64::consts::FRAC_1_SQRT_2;
    approx_eq!(mid.x, s2, tol);
    approx_eq!(mid.y, 0.0, tol);
    approx_eq!(mid.z, s2, tol);
}

#[test]
fn arc_arc_length() {
    // Quarter arc, R=2.0: arc length = R * |θ2 - θ1| = 2 * π/2 = π
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 1, 2.0, 0.0, 90.0, 0.001)]), None).unwrap();
    approx_eq!(mesh.segments[0].curve.arc_length(), PI, 1e-10);
}

#[test]
fn arc_speed_is_constant() {
    // For a circular arc, |dr/dσ| = R * |dθ/dσ| = R * |θ2 - θ1|, constant.
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 1, 1.0, 0.0, 90.0, 0.001)]), None).unwrap();
    let expected_speed = 1.0 * (PI / 2.0); // R * |Δθ|
    let tol = 1e-10;
    approx_eq!(mesh.segments[0].curve.speed(0.0), expected_speed, tol);
    approx_eq!(mesh.segments[0].curve.speed(0.5), expected_speed, tol);
    approx_eq!(mesh.segments[0].curve.speed(1.0), expected_speed, tol);
}

#[test]
fn arc_tangent_perpendicular_to_radius() {
    // For a circular arc, the tangent is always perpendicular to the radius vector.
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 1, 1.0, 0.0, 90.0, 0.001)]), None).unwrap();
    let curve = &mesh.segments[0].curve;
    let tol = 1e-10;
    for sigma in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let r = curve.evaluate(sigma); // center is origin, so r IS the radius vector
        let t = curve.tangent(sigma);
        let dot = r.dot(&t);
        approx_eq!(dot, 0.0, tol);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helix: evaluate(0) == start, evaluate(1) == end
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn helix_evaluate_endpoints() {
    // 1-turn helix: spacing=1turn, total_length=1.0, R=0.1, 10 segments.
    let (mesh, _) =
        build_mesh(free_space(vec![gh(1, 10, 1.0, 1.0, 0.1, 0.1, 0.001)]), None).unwrap();

    let tol = 1e-10;
    for seg in &mesh.segments {
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }
}

#[test]
fn helix_evaluate_midpoint() {
    // Single-segment, 1-turn uniform helix: R=0.1, HL=1.0
    let (mesh, _) =
        build_mesh(free_space(vec![gh(1, 1, 1.0, 1.0, 0.1, 0.1, 0.001)]), None).unwrap();

    // Midpoint at τ=0.5: angle = 2π*1*0.5 = π
    // r(0.5) = (0.1*cos(π), 0.1*sin(π), 1.0*0.5) = (-0.1, ~0, 0.5)
    let mid = mesh.segments[0].curve.evaluate(0.5);
    let tol = 1e-10;
    approx_eq!(mid.x, -0.1, tol);
    approx_eq!(mid.y, 0.0, 1e-9); // sin(π) ≈ 0, not exactly 0
    approx_eq!(mid.z, 0.5, tol);
}

#[test]
fn helix_arc_length_uniform() {
    // Uniform helix: R=0.1, 1 turn, HL=1.0, 1 segment.
    // Exact arc length = sqrt((2πR)² + HL²) = sqrt((2π*0.1)² + 1²)
    let (mesh, _) =
        build_mesh(free_space(vec![gh(1, 1, 1.0, 1.0, 0.1, 0.1, 0.001)]), None).unwrap();

    let expected = ((2.0 * PI * 0.1_f64).powi(2) + 1.0_f64.powi(2)).sqrt();
    // Numerical integration with 16 points should be close.
    approx_eq!(mesh.segments[0].curve.arc_length(), expected, 1e-6);
}

// ─────────────────────────────────────────────────────────────────────────────
// Evaluation after GM transform
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn arc_evaluate_after_gm_rotation() {
    // Arc in XZ plane, then rotated 90° about X axis → should end up in XY plane.
    let gm = GmOperation {
        tag: 0,
        n_copies: 0,
        rot_x: 90.0,
        rot_y: 0.0,
        rot_z: 0.0,
        trans_x: 0.0,
        trans_y: 0.0,
        trans_z: 0.0,
        tag_increment: 0,
    };
    let (mesh, _) = build_mesh(with_gm(vec![ga(1, 4, 0.5, 0.0, 180.0, 0.001)], gm), None).unwrap();

    let tol = 1e-10;
    for seg in &mesh.segments {
        // evaluate(0) and evaluate(1) must still match the transformed endpoints.
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }

    // After 90° rotation about X, original z becomes -y.
    // Original arc at θ=0: (0.5, 0, 0) → (0.5, 0, 0) (no change, z was 0).
    // Original arc at θ=90°: (0, 0, 0.5) → (0, -0.5, 0).
    let seg1_end = mesh.segments[1].end(); // θ=90° point
    approx_eq!(seg1_end.x, 0.0, tol);
    approx_eq!(seg1_end.y, -0.5, tol);
    approx_eq!(seg1_end.z, 0.0, tol);

    // Midpoint of segment 0 (θ=22.5°): original (0.5*cos22.5, 0, 0.5*sin22.5)
    // After Rx(90°): (x, -z, y) = (0.5*cos22.5, -0.5*sin22.5, 0)
    let mid = mesh.segments[0].curve.evaluate(0.5);
    let theta = (22.5_f64).to_radians();
    approx_eq!(mid.x, 0.5 * theta.cos(), tol);
    approx_eq!(mid.y, -0.5 * theta.sin(), tol);
    approx_eq!(mid.z, 0.0, tol);
}

#[test]
fn arc_evaluate_after_gm_translation() {
    // Arc translated by (1, 2, 3).
    let gm = GmOperation {
        tag: 0,
        n_copies: 0,
        rot_x: 0.0,
        rot_y: 0.0,
        rot_z: 0.0,
        trans_x: 1.0,
        trans_y: 2.0,
        trans_z: 3.0,
        tag_increment: 0,
    };
    let (mesh, _) = build_mesh(with_gm(vec![ga(1, 2, 1.0, 0.0, 90.0, 0.001)], gm), None).unwrap();

    let tol = 1e-10;
    for seg in &mesh.segments {
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }

    // Segment 0 start: original (1, 0, 0), translated → (2, 2, 3)
    let s = mesh.segments[0].start();
    approx_eq!(s.x, 2.0, tol);
    approx_eq!(s.y, 2.0, tol);
    approx_eq!(s.z, 3.0, tol);
}

#[test]
fn helix_evaluate_after_gm_rotation() {
    // Helix along z-axis, then rotated 90° about Y → helix along x-axis.
    let gm = GmOperation {
        tag: 0,
        n_copies: 0,
        rot_x: 0.0,
        rot_y: 90.0,
        rot_z: 0.0,
        trans_x: 0.0,
        trans_y: 0.0,
        trans_z: 0.0,
        tag_increment: 0,
    };
    let (mesh, _) =
        build_mesh(with_gm(vec![gh(1, 4, 1.0, 1.0, 0.1, 0.1, 0.001)], gm), None).unwrap();

    let tol = 1e-10;
    for seg in &mesh.segments {
        vec3_approx_eq(seg.curve.evaluate(0.0), seg.start(), tol);
        vec3_approx_eq(seg.curve.evaluate(1.0), seg.end(), tol);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Continuity: evaluate(1.0) of seg k == evaluate(0.0) of seg k+1
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn arc_evaluate_continuity() {
    let (mesh, _) = build_mesh(free_space(vec![ga(1, 8, 0.5, 0.0, 360.0, 0.001)]), None).unwrap();

    let tol = 1e-12;
    for k in 0..mesh.segments.len() - 1 {
        let end_k = mesh.segments[k].curve.evaluate(1.0);
        let start_k1 = mesh.segments[k + 1].curve.evaluate(0.0);
        vec3_approx_eq(end_k, start_k1, tol);
    }
}

#[test]
fn helix_evaluate_continuity() {
    let (mesh, _) =
        build_mesh(free_space(vec![gh(1, 10, 1.0, 1.0, 0.1, 0.1, 0.001)]), None).unwrap();

    let tol = 1e-12;
    for k in 0..mesh.segments.len() - 1 {
        let end_k = mesh.segments[k].curve.evaluate(1.0);
        let start_k1 = mesh.segments[k + 1].curve.evaluate(0.0);
        vec3_approx_eq(end_k, start_k1, tol);
    }
}
