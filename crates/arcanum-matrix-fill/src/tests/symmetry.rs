//! V-SYM: Matrix symmetry validation cases.

use super::helpers;
use crate::{fill_impedance_matrix, MatrixFillConfig};

const FREQ: f64 = 300e6;
const SEG_LENGTH: f64 = 0.05;
const A_MOD: f64 = 0.005;

/// V-SYM-001 — Symmetry of Two-Segment Dipole Matrix.
#[test]
fn v_sym_001_two_segment_symmetry() {
    let mesh = helpers::straight_wire_mesh(2, SEG_LENGTH, A_MOD);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let diff = (z.read(0, 1) - z.read(1, 0)).abs();
    assert!(
        diff < 1e-15,
        "V-SYM-001: |Z[0,1] - Z[1,0]| = {diff}, expected < ε_machine"
    );
}

/// V-SYM-002 — Symmetry of 11-Segment Dipole Matrix.
#[test]
fn v_sym_002_eleven_segment_symmetry() {
    let mesh = helpers::straight_wire_mesh(11, SEG_LENGTH, A_MOD);
    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let mut max_diff = 0.0f64;
    for m in 0..11 {
        for n in 0..11 {
            let diff = (z.read(m, n) - z.read(n, m)).abs();
            max_diff = max_diff.max(diff);
        }
    }
    assert!(
        max_diff < 1e-15,
        "V-SYM-002: max |Z[m,n] - Z[n,m]| = {max_diff}, expected < ε_machine"
    );
}

/// V-SYM-003 — Symmetry of Mixed Geometry Matrix (straight + arc, separated).
#[test]
fn v_sym_003_mixed_geometry_symmetry() {
    use arcanum_geometry::mesh::{
        ArcParams, CurveParams, GroundDescriptor, GroundType, LinearParams, Material, Mesh,
        Segment, TagMap,
    };
    use nalgebra::{Matrix3, Vector3};
    use std::f64::consts::PI;

    // Straight segment along z-axis.
    let seg_straight = Segment {
        curve: CurveParams::Linear(LinearParams {
            start: Vector3::new(0.0, 0.0, 0.0),
            end: Vector3::new(0.0, 0.0, SEG_LENGTH),
        }),
        wire_radius: A_MOD,
        material: Material::PEC,
        tag: 1,
        segment_index: 0,
        wire_index: 0,
        is_image: false,
    };

    // Arc segment offset in y, subtending 90° with arc length ≈ SEG_LENGTH.
    // arc_length = R × Δθ → R = arc_length / Δθ = 0.05 / (π/2) ≈ 0.0318
    let arc_radius = SEG_LENGTH / (PI / 2.0);
    let theta1: f64 = 0.0;
    let theta2 = PI / 2.0;
    let center = Vector3::new(0.0, 0.5, 0.0); // separated from straight segment
    let start = center + Vector3::new(arc_radius * theta1.cos(), 0.0, arc_radius * theta1.sin());
    let end = center + Vector3::new(arc_radius * theta2.cos(), 0.0, arc_radius * theta2.sin());

    let seg_arc = Segment {
        curve: CurveParams::Arc(ArcParams {
            radius: arc_radius,
            theta1,
            theta2,
            rotation: Matrix3::identity(),
            center,
            start,
            end,
        }),
        wire_radius: A_MOD,
        material: Material::PEC,
        tag: 2,
        segment_index: 1,
        wire_index: 1,
        is_image: false,
    };

    let mesh = Mesh {
        segments: vec![seg_straight, seg_arc],
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

    let config = MatrixFillConfig::default();
    let z = fill_impedance_matrix(&mesh, FREQ, &config);

    let diff = (z.read(0, 1) - z.read(1, 0)).abs();
    assert!(
        diff < 1e-15,
        "V-SYM-003: |Z[0,1] - Z[1,0]| = {diff}, expected < ε_machine"
    );
}
