// warnings.rs — V-WARN validation cases (Step 10)

use arcanum_nec_import::{
    GeometricGround, GeometryTransforms, MeshInput, StraightWire, WireDescription,
};

use crate::build_mesh;
use crate::errors::GeometryWarningKind;

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-001 — WireInGroundPlane warning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_001_wire_in_ground_plane_warning() {
    use arcanum_nec_import::GroundType as NecGroundType;

    // Horizontal wire at z=0 above PEC ground.
    let (_, warnings) = build_mesh(
        MeshInput {
            wires: vec![WireDescription::Straight(StraightWire {
                tag: 1,
                segment_count: 2,
                x1: -0.5,
                y1: 0.0,
                z1: 0.0,
                x2: 0.5,
                y2: 0.0,
                z2: 0.0,
                radius: 0.001,
            })],
            ground: GeometricGround {
                ground_type: NecGroundType::PEC,
            },
            gpflag: 1,
            transforms: GeometryTransforms::default(),
        },
        None,
    )
    .expect("build_mesh must succeed");

    let warn_vec = warnings.into_vec();
    assert!(!warn_vec.is_empty(), "expected at least one warning");
    let has_ground_warn = warn_vec
        .iter()
        .any(|w| w.kind == GeometryWarningKind::WireInGroundPlane);
    assert!(
        has_ground_warn,
        "expected WireInGroundPlane warning, got: {:?}",
        warn_vec
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-002 — NearCoincidentEndpoints warning
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_002_near_coincident_endpoints_warning() {
    // Two wires whose endpoints almost share a point.
    // GW 1 1 0.0 0.0 0.0  1.0 0.0 0.0   wire_radius=0.001
    // GW 2 1 1.0 0.0 5e-5 2.0 0.0 5e-5  wire_radius=0.001
    //
    // Gap between wire 1 end (1.0, 0, 0) and wire 2 start (1.0, 0, 5e-5) = 5e-5 m.
    // ε = min(0.001, 0.001) × 0.01 = 1e-5 m.
    // 10ε = 1e-4 m.
    // gap (5e-5) is in [ε, 10ε] → near-coincident warning, no junction.
    let gap = 5e-5_f64;
    let (mesh, warnings) = build_mesh(
        MeshInput {
            wires: vec![
                WireDescription::Straight(StraightWire {
                    tag: 1,
                    segment_count: 1,
                    x1: 0.0,
                    y1: 0.0,
                    z1: 0.0,
                    x2: 1.0,
                    y2: 0.0,
                    z2: 0.0,
                    radius: 0.001,
                }),
                WireDescription::Straight(StraightWire {
                    tag: 2,
                    segment_count: 1,
                    x1: 1.0,
                    y1: 0.0,
                    z1: gap,
                    x2: 2.0,
                    y2: 0.0,
                    z2: gap,
                    radius: 0.001,
                }),
            ],
            ground: GeometricGround::default(),
            gpflag: 0,
            transforms: GeometryTransforms::default(),
        },
        None,
    )
    .expect("build_mesh must succeed");

    // Mesh is produced — warning, not error.
    assert_eq!(mesh.segments.len(), 2);

    // NearCoincidentEndpoints warning emitted.
    let warn_vec = warnings.into_vec();
    assert!(!warn_vec.is_empty(), "expected at least one warning");
    let has_near_warn = warn_vec
        .iter()
        .any(|w| w.kind == GeometryWarningKind::NearCoincidentEndpoints);
    assert!(
        has_near_warn,
        "expected NearCoincidentEndpoints warning, got: {:?}",
        warn_vec
    );

    // No junction created at the near-coincident gap.
    assert!(
        mesh.junctions.is_empty(),
        "near-coincident gap must not form a junction"
    );
}
