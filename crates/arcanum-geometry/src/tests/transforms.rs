// transforms.rs — V-TRF validation cases (Step 5)

use arcanum_nec_import::{
    GeometricGround, GeometryTransforms, GmOperation, MeshInput, StraightWire, WireDescription,
};

use crate::build_mesh;

use super::approx_eq;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn gw(
    tag: u32,
    n: u32,
    x1: f64, y1: f64, z1: f64,
    x2: f64, y2: f64, z2: f64,
    radius: f64,
) -> WireDescription {
    WireDescription::Straight(StraightWire { tag, segment_count: n, x1, y1, z1, x2, y2, z2, radius })
}

fn input(wires: Vec<WireDescription>, transforms: GeometryTransforms) -> MeshInput {
    MeshInput {
        wires,
        ground: GeometricGround::default(),
        gpflag: 0,
        transforms,
    }
}

fn no_rotation_no_translation() -> GmOperation {
    GmOperation {
        tag: 0,
        n_copies: 0,
        rot_x: 0.0,
        rot_y: 0.0,
        rot_z: 0.0,
        trans_x: 0.0,
        trans_y: 0.0,
        trans_z: 0.0,
        tag_increment: 0,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-TRF-001 — GS global scale
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_trf_001_gs_global_scale() {
    // GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.01
    // GS 0 0 0.5  — scale all coordinates by 0.5; wire radius NOT scaled
    let transforms = GeometryTransforms {
        gs_scale: Some(0.5),
        gm_ops: vec![],
    };
    let (mesh, _) = build_mesh(
        input(vec![gw(1, 4, 0.0, 0.0, -0.25, 0.0, 0.0, 0.25, 0.01)], transforms),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 4);

    // z range should now be -0.125 to 0.125 (scaled by 0.5).
    let tol = 1e-12;
    approx_eq!(mesh.segments[0].start().z, -0.125, tol);
    approx_eq!(mesh.segments[3].end().z, 0.125, tol);

    // x and y remain 0.
    for seg in &mesh.segments {
        approx_eq!(seg.start().x, 0.0, tol);
        approx_eq!(seg.start().y, 0.0, tol);
        approx_eq!(seg.end().x, 0.0, tol);
        approx_eq!(seg.end().y, 0.0, tol);
    }

    // Wire radius must NOT be scaled — still 0.01.
    for seg in &mesh.segments {
        approx_eq!(seg.wire_radius, 0.01, tol);
    }

    // Total wire length: 0.25 m (was 0.5 m, scaled by 0.5).
    let total_z = mesh.segments[3].end().z - mesh.segments[0].start().z;
    approx_eq!(total_z, 0.25, tol);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-TRF-002 — GM translation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_trf_002_gm_translation() {
    // GW 1 2 0.0 0.0 -0.25 0.0 0.0 0.25 0.001
    // GM 0 0 0.0 0.0 0.0 1.0 0.0 0.0  — translate all by (+1, 0, 0)
    let mut gm = no_rotation_no_translation();
    gm.trans_x = 1.0;

    let transforms = GeometryTransforms {
        gs_scale: None,
        gm_ops: vec![gm],
    };

    let (mesh, _) = build_mesh(
        input(vec![gw(1, 2, 0.0, 0.0, -0.25, 0.0, 0.0, 0.25, 0.001)], transforms),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 2);

    let tol = 1e-12;

    // All x-coordinates should be +1.0.
    approx_eq!(mesh.segments[0].start().x, 1.0, tol);
    approx_eq!(mesh.segments[0].end().x,   1.0, tol);
    approx_eq!(mesh.segments[1].start().x, 1.0, tol);
    approx_eq!(mesh.segments[1].end().x,   1.0, tol);

    // y and z unchanged.
    approx_eq!(mesh.segments[0].start().y, 0.0, tol);
    approx_eq!(mesh.segments[0].start().z, -0.25, tol);
    approx_eq!(mesh.segments[1].end().z,   0.25, tol);

    // Explicit segment coordinates from validation.md:
    // seg 0 start: (1.0, 0.0, -0.25)
    approx_eq!(mesh.segments[0].start().x, 1.0, tol);
    approx_eq!(mesh.segments[0].start().z, -0.25, tol);
    // seg 1 end: (1.0, 0.0, 0.25)
    approx_eq!(mesh.segments[1].end().x, 1.0, tol);
    approx_eq!(mesh.segments[1].end().z, 0.25, tol);

    // Wire radius unchanged.
    approx_eq!(mesh.segments[0].wire_radius, 0.001, tol);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-TRF-003 — GM rotation about z-axis (90°)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_trf_003_gm_rotation_z_90deg() {
    // GW 1 1 1.0 0.0 0.0  1.0 0.0 1.0 0.001
    // GM 0 0 0.0 0.0 90.0 0.0 0.0 0.0  — rotate all 90° around z-axis
    // After 90° Rz: (x, y, z) → (-y, x, z)
    // (1, 0, 0) → (0, 1, 0)
    // (1, 0, 1) → (0, 1, 1)
    let mut gm = no_rotation_no_translation();
    gm.rot_z = 90.0;

    let transforms = GeometryTransforms {
        gs_scale: None,
        gm_ops: vec![gm],
    };

    let (mesh, _) = build_mesh(
        input(vec![gw(1, 1, 1.0, 0.0, 0.0, 1.0, 0.0, 1.0, 0.001)], transforms),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.segments.len(), 1);

    let tol = 1e-9;

    // start: (1,0,0) → (0, 1, 0)
    approx_eq!(mesh.segments[0].start().x, 0.0, tol);
    approx_eq!(mesh.segments[0].start().y, 1.0, tol);
    approx_eq!(mesh.segments[0].start().z, 0.0, tol);

    // end: (1,0,1) → (0, 1, 1)
    approx_eq!(mesh.segments[0].end().x, 0.0, tol);
    approx_eq!(mesh.segments[0].end().y, 1.0, tol);
    approx_eq!(mesh.segments[0].end().z, 1.0, tol);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-TRF-004 — GM n_copies > 0: generate translated copies
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_trf_004_gm_copies_translation() {
    // GW 1 1 0.0 0.0 0.0  1.0 0.0 0.0 0.001
    // GM 0 2 0.0 0.0 0.0 0.0 1.0 0.0  ITS=1
    // n_copies=2, trans_y=1.0 → 2 additional copies at y=1 and y=2
    let gm = GmOperation {
        tag: 0,
        n_copies: 2,
        rot_x: 0.0,
        rot_y: 0.0,
        rot_z: 0.0,
        trans_x: 0.0,
        trans_y: 1.0,
        trans_z: 0.0,
        tag_increment: 1,
    };

    let transforms = GeometryTransforms {
        gs_scale: None,
        gm_ops: vec![gm],
    };

    let (mesh, _) = build_mesh(
        input(vec![gw(1, 1, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.001)], transforms),
        None,
    )
    .expect("build_mesh failed");

    // Original + 2 copies = 3 segments total.
    assert_eq!(mesh.segments.len(), 3);

    let tol = 1e-12;

    // Original (tag 1): y = 0.
    approx_eq!(mesh.segments[0].start().y, 0.0, tol);
    assert_eq!(mesh.segments[0].tag, 1);

    // Copy 1 (tag 2): y = 1.
    approx_eq!(mesh.segments[1].start().y, 1.0, tol);
    assert_eq!(mesh.segments[1].tag, 2);

    // Copy 2 (tag 3): y = 2.
    approx_eq!(mesh.segments[2].start().y, 2.0, tol);
    assert_eq!(mesh.segments[2].tag, 3);

    // Tag map: tag 2 and tag 3 registered.
    assert_eq!(mesh.tag_map.get(2), Some((1, 1)));
    assert_eq!(mesh.tag_map.get(3), Some((2, 2)));
}
