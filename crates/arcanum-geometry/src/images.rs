// images.rs — PEC ground plane image generation (Step 7)
//
// For each real segment not lying in the z = 0 plane:
//   - Create an image segment with z-coordinates negated
//   - Image segments are flagged is_image = true
//   - Image segments are appended after all real segments
//
// Wires entirely in the z = 0 plane produce no image; a warning is emitted.

use crate::errors::{GeometryWarning, GeometryWarningKind, GeometryWarnings};
use nalgebra::Matrix3;

use crate::mesh::{ArcParams, CurveParams, HelixParams, LinearParams, Segment};

/// Diagonal matrix that reflects z: diag(1, 1, -1).
fn z_reflect_matrix() -> Matrix3<f64> {
    Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, -1.0)
}

pub(crate) fn generate(segments: &mut Vec<Segment>, warnings: &mut GeometryWarnings) {
    let real_count = segments.len();
    let mut image_segs: Vec<Segment> = Vec::new();

    for seg in segments[..real_count].iter() {
        let start_z = seg.start().z;
        let end_z = seg.end().z;

        // Detect wire entirely in the z = 0 ground plane.
        if start_z == 0.0 && end_z == 0.0 {
            warnings.push(GeometryWarning::new(
                GeometryWarningKind::WireInGroundPlane,
                format!(
                    "segment {} (tag={}) lies in z=0 ground plane; no image generated",
                    seg.segment_index, seg.tag
                ),
            ));
            continue;
        }

        // Build image segment: negate z-coordinates.
        let image_curve = reflect_z(&seg.curve);
        let base_index = real_count + image_segs.len();

        image_segs.push(Segment {
            curve: image_curve,
            wire_radius: seg.wire_radius,
            material: seg.material.clone(),
            tag: seg.tag,
            segment_index: base_index,
            wire_index: seg.wire_index,
            is_image: true,
        });
    }

    segments.extend(image_segs);
}

/// Reflect a CurveParams through the z = 0 plane: z → -z.
fn reflect_z(curve: &CurveParams) -> CurveParams {
    match curve {
        CurveParams::Linear(p) => {
            let mut start = p.start;
            let mut end = p.end;
            start.z = -start.z;
            end.z = -end.z;
            CurveParams::Linear(LinearParams { start, end })
        }
        CurveParams::Arc(p) => {
            let mut start = p.start;
            let mut end = p.end;
            start.z = -start.z;
            end.z = -end.z;
            let rz = z_reflect_matrix();
            CurveParams::Arc(ArcParams {
                radius: p.radius,
                theta1: -p.theta1,
                theta2: -p.theta2,
                rotation: rz * p.rotation,
                center: {
                    let mut c = p.center;
                    c.z = -c.z;
                    c
                },
                start,
                end,
            })
        }
        CurveParams::Helix(p) => {
            let mut start = p.start;
            let mut end = p.end;
            start.z = -start.z;
            end.z = -end.z;
            let rz = z_reflect_matrix();
            CurveParams::Helix(HelixParams {
                radius_start: p.radius_start,
                radius_end: p.radius_end,
                total_length: p.total_length,
                n_turns: p.n_turns,
                n_segments: p.n_segments,
                segment_index: p.segment_index,
                rotation: rz * p.rotation,
                center: {
                    let mut c = p.center;
                    c.z = -c.z;
                    c
                },
                start,
                end,
            })
        }
    }
}
