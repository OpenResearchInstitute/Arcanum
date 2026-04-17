// ground.rs — V-GND validation cases (Step 7)

use arcanum_nec_import::{
    GeometricGround, GeometryTransforms, GroundElectrical, GroundModel,
    GroundType as NecGroundType, MeshInput, StraightWire, WireDescription,
};

use crate::build_mesh;
use crate::mesh::GroundType;

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

fn input_with_ground(wires: Vec<WireDescription>, ground_type: NecGroundType) -> MeshInput {
    MeshInput {
        wires,
        ground: GeometricGround { ground_type },
        gpflag: 0,
        transforms: GeometryTransforms::default(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// V-GND-001 — No ground card
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_gnd_001_no_ground() {
    // GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001 / GE 0
    let (mesh, _) = build_mesh(
        input_with_ground(
            vec![gw(1, 4, 0.0, 0.0, -0.25, 0.0, 0.0, 0.25, 0.001)],
            NecGroundType::FreeSpace,
        ),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.ground.ground_type, GroundType::None);
    assert!(!mesh.ground.images_generated);
    assert_eq!(mesh.real_segment_count(), 4);
    assert_eq!(mesh.image_segment_count(), 0);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-GND-002 — PEC ground, image generation
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_gnd_002_pec_ground_image_generation() {
    // GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001 / GN 1 / GE 1
    // A vertical wire from z=0 to z=0.5 above PEC ground.
    let (mesh, _) = build_mesh(
        input_with_ground(
            vec![gw(1, 4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.001)],
            NecGroundType::PEC,
        ),
        None,
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.ground.ground_type, GroundType::PEC);
    assert!(mesh.ground.images_generated);

    // 4 real + 4 image = 8 total.
    // (Note: the bottom segment starts at z=0 — it will get an image since z_end != 0.)
    let real_count = mesh.real_segment_count();
    let image_count = mesh.image_segment_count();
    assert_eq!(
        real_count + image_count,
        8,
        "expected 8 total segments (4 real + 4 image)"
    );
    assert_eq!(real_count, 4);
    assert_eq!(image_count, 4);

    let tol = 1e-12;

    // Each image segment has z-coordinates negated.
    for k in 0..4usize {
        let real = &mesh.segments[k];
        let image = &mesh.segments[4 + k];
        assert!(image.is_image);
        approx_eq!(image.start().x, real.start().x, tol);
        approx_eq!(image.start().y, real.start().y, tol);
        approx_eq!(image.start().z, -real.start().z, tol);
        approx_eq!(image.end().x, real.end().x, tol);
        approx_eq!(image.end().y, real.end().y, tol);
        approx_eq!(image.end().z, -real.end().z, tol);
        approx_eq!(image.wire_radius, real.wire_radius, tol);
        assert_eq!(image.tag, real.tag);
    }

    // One junction at z = 0: real segment 0 Start and image segment 4 Start share
    // the ground-plane contact point. Adjacent image segments are the same wire and
    // are skipped by the intra-wire adjacency rule.
    assert_eq!(mesh.junctions.len(), 1);
    assert!(!mesh.junctions[0].is_self_loop);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-GND-003 — Lossy ground parameters stored, no images
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_gnd_003_lossy_ground_params_stored() {
    // GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001 / GN 2 0 0 0 13.0 0.005 / GE 1
    let ground_elec = GroundElectrical {
        permittivity: 13.0,
        conductivity: 0.005,
        model: GroundModel::ReflectionCoeff,
    };

    let (mesh, _) = build_mesh(
        input_with_ground(
            vec![gw(1, 4, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5, 0.001)],
            NecGroundType::Lossy,
        ),
        Some(ground_elec),
    )
    .expect("build_mesh failed");

    assert_eq!(mesh.ground.ground_type, GroundType::Lossy);
    assert!(
        !mesh.ground.images_generated,
        "lossy ground must not generate images"
    );
    assert_eq!(mesh.image_segment_count(), 0);
    assert_eq!(mesh.real_segment_count(), 4);

    // Electrical parameters stored for Phase 2.
    approx_eq!(
        mesh.ground.conductivity.expect("conductivity missing"),
        0.005
    );
    approx_eq!(
        mesh.ground.permittivity.expect("permittivity missing"),
        13.0
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-GND-004 — Wire in ground plane (no self-image, warning emitted)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_gnd_004_wire_in_ground_plane() {
    // GW 1 4 -0.25 0.0 0.0 0.25 0.0 0.0 0.001 / GN 1 / GE 1
    // All z-coordinates = 0.0 → wire lies in ground plane.
    let (mesh, warnings) = build_mesh(
        input_with_ground(
            vec![gw(1, 4, -0.25, 0.0, 0.0, 0.25, 0.0, 0.0, 0.001)],
            NecGroundType::PEC,
        ),
        None,
    )
    .expect("build_mesh failed");

    // No image segments — wire is its own image.
    assert_eq!(
        mesh.image_segment_count(),
        0,
        "no images for wire in ground plane"
    );
    assert_eq!(mesh.real_segment_count(), 4);
    // No junctions — single wire, no images, no cross-wire connections.
    assert!(mesh.junctions.is_empty());

    // Warning emitted.
    let warn_vec = warnings.into_vec();
    assert!(!warn_vec.is_empty(), "expected WireInGroundPlane warning");
    use crate::errors::GeometryWarningKind;
    let has_warn = warn_vec
        .iter()
        .any(|w| w.kind == GeometryWarningKind::WireInGroundPlane);
    assert!(has_warn, "expected WireInGroundPlane warning kind");
}
