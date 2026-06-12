//! Shared test helpers for programmatic mesh construction.

use arcanum_geometry::mesh::{
    ArcParams, CurveParams, GroundDescriptor, GroundType, HelixParams, LinearParams, Material,
    Mesh, Segment, TagMap,
};
use nalgebra::{Matrix3, Vector3};
use std::f64::consts::PI;

/// Build a straight wire along the z-axis with N equal-length segments.
pub fn straight_wire_mesh(n_segments: usize, seg_length: f64, wire_radius: f64) -> Mesh {
    let segments: Vec<Segment> = (0..n_segments)
        .map(|i| {
            let z0 = i as f64 * seg_length;
            let z1 = z0 + seg_length;
            Segment {
                curve: CurveParams::Linear(LinearParams {
                    start: Vector3::new(0.0, 0.0, z0),
                    end: Vector3::new(0.0, 0.0, z1),
                }),
                wire_radius,
                material: Material::PEC,
                tag: 1,
                segment_index: i,
                wire_index: 0,
                is_image: false,
            }
        })
        .collect();

    empty_mesh(segments)
}

/// Build a single arc segment in the XZ plane.
///
/// The arc has the given `arc_radius`, starts at angle `theta1` and ends at `theta2`.
pub fn single_arc_segment(arc_radius: f64, theta1: f64, theta2: f64, wire_radius: f64) -> Mesh {
    let start = Vector3::new(arc_radius * theta1.cos(), 0.0, arc_radius * theta1.sin());
    let end = Vector3::new(arc_radius * theta2.cos(), 0.0, arc_radius * theta2.sin());

    let seg = Segment {
        curve: CurveParams::Arc(ArcParams {
            radius: arc_radius,
            theta1,
            theta2,
            rotation: Matrix3::identity(),
            center: Vector3::zeros(),
            start,
            end,
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 0,
        wire_index: 0,
        is_image: false,
    };

    empty_mesh(vec![seg])
}

/// Build two arc segments separated in space (non-adjacent).
pub fn two_arc_segments_separated(
    arc_radius: f64,
    subtend_angle: f64,
    wire_radius: f64,
    separation: f64,
) -> Mesh {
    let theta1_a: f64 = 0.0;
    let theta2_a = subtend_angle;
    let start_a = Vector3::new(arc_radius * theta1_a.cos(), 0.0, arc_radius * theta1_a.sin());
    let end_a = Vector3::new(arc_radius * theta2_a.cos(), 0.0, arc_radius * theta2_a.sin());

    let seg_a = Segment {
        curve: CurveParams::Arc(ArcParams {
            radius: arc_radius,
            theta1: theta1_a,
            theta2: theta2_a,
            rotation: Matrix3::identity(),
            center: Vector3::zeros(),
            start: start_a,
            end: end_a,
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 0,
        wire_index: 0,
        is_image: false,
    };

    // Second arc segment, translated along y by `separation`.
    let center_b = Vector3::new(0.0, separation, 0.0);
    let start_b = center_b + Vector3::new(arc_radius * theta1_a.cos(), 0.0, arc_radius * theta1_a.sin());
    let end_b = center_b + Vector3::new(arc_radius * theta2_a.cos(), 0.0, arc_radius * theta2_a.sin());

    let seg_b = Segment {
        curve: CurveParams::Arc(ArcParams {
            radius: arc_radius,
            theta1: theta1_a,
            theta2: theta2_a,
            rotation: Matrix3::identity(),
            center: center_b,
            start: start_b,
            end: end_b,
        }),
        wire_radius,
        material: Material::PEC,
        tag: 2,
        segment_index: 1,
        wire_index: 1,
        is_image: false,
    };

    empty_mesh(vec![seg_a, seg_b])
}

/// Build a single helix segment.
///
/// Creates one segment of an 8-segment helix with given parameters.
pub fn single_helix_segment(
    helix_radius: f64,
    total_length: f64,
    n_turns: f64,
    seg_index: u32,
    n_segments: u32,
    wire_radius: f64,
) -> Mesh {
    let params = HelixParams {
        radius_start: helix_radius,
        radius_end: helix_radius,
        total_length,
        n_turns,
        n_segments,
        segment_index: seg_index,
        rotation: Matrix3::identity(),
        center: Vector3::zeros(),
        start: helix_point(helix_radius, total_length, n_turns, n_segments, seg_index),
        end: helix_point(helix_radius, total_length, n_turns, n_segments, seg_index + 1),
    };

    let seg = Segment {
        curve: CurveParams::Helix(params),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: seg_index as usize,
        wire_index: 0,
        is_image: false,
    };

    empty_mesh(vec![seg])
}

/// Build two helix segments (first and last of an 8-segment helix).
pub fn two_helix_segments(
    helix_radius: f64,
    total_length: f64,
    n_turns: f64,
    wire_radius: f64,
) -> Mesh {
    let n_segments: u32 = 8;

    let seg0 = Segment {
        curve: CurveParams::Helix(HelixParams {
            radius_start: helix_radius,
            radius_end: helix_radius,
            total_length,
            n_turns,
            n_segments,
            segment_index: 0,
            rotation: Matrix3::identity(),
            center: Vector3::zeros(),
            start: helix_point(helix_radius, total_length, n_turns, n_segments, 0),
            end: helix_point(helix_radius, total_length, n_turns, n_segments, 1),
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 0,
        wire_index: 0,
        is_image: false,
    };

    let seg7 = Segment {
        curve: CurveParams::Helix(HelixParams {
            radius_start: helix_radius,
            radius_end: helix_radius,
            total_length,
            n_turns,
            n_segments,
            segment_index: 7,
            rotation: Matrix3::identity(),
            center: Vector3::zeros(),
            start: helix_point(helix_radius, total_length, n_turns, n_segments, 7),
            end: helix_point(helix_radius, total_length, n_turns, n_segments, 8),
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 1,
        wire_index: 0,
        is_image: false,
    };

    empty_mesh(vec![seg0, seg7])
}

/// Compute a point on a uniform-radius helix at segment boundary `idx` out of `n_seg`.
fn helix_point(radius: f64, total_length: f64, n_turns: f64, n_seg: u32, idx: u32) -> Vector3<f64> {
    let t = idx as f64 / n_seg as f64;
    let theta = 2.0 * PI * n_turns * t;
    let z = total_length * t;
    Vector3::new(radius * theta.cos(), radius * theta.sin(), z)
}

/// Build a mesh with two parallel segments separated by perpendicular distance d.
///
/// A spacer segment is inserted between them so that the two test segments
/// are at mesh indices 0 and 2 (not adjacent, classified as regular elements).
/// The mutual impedance of interest is `Z[0, 2]`.
pub fn two_parallel_segments(seg_length: f64, wire_radius: f64, separation: f64) -> Mesh {
    let seg_a = Segment {
        curve: CurveParams::Linear(LinearParams {
            start: Vector3::new(0.0, 0.0, 0.0),
            end: Vector3::new(0.0, 0.0, seg_length),
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 0,
        wire_index: 0,
        is_image: false,
    };

    // Spacer segment (continuation of wire A) — ensures seg_a and seg_b
    // are not adjacent in the mesh.
    let seg_spacer = Segment {
        curve: CurveParams::Linear(LinearParams {
            start: Vector3::new(0.0, 0.0, seg_length),
            end: Vector3::new(0.0, 0.0, 2.0 * seg_length),
        }),
        wire_radius,
        material: Material::PEC,
        tag: 1,
        segment_index: 1,
        wire_index: 0,
        is_image: false,
    };

    let seg_b = Segment {
        curve: CurveParams::Linear(LinearParams {
            start: Vector3::new(separation, 0.0, 0.0),
            end: Vector3::new(separation, 0.0, seg_length),
        }),
        wire_radius,
        material: Material::PEC,
        tag: 2,
        segment_index: 2,
        wire_index: 1,
        is_image: false,
    };

    empty_mesh(vec![seg_a, seg_spacer, seg_b])
}

fn empty_mesh(segments: Vec<Segment>) -> Mesh {
    Mesh {
        segments,
        junctions: vec![],
        endpoint_junction: vec![],
        ground: GroundDescriptor {
            ground_type: GroundType::None,
            conductivity: None,
            permittivity: None,
            images_generated: false,
        },
        tag_map: TagMap::default(),
    }
}
