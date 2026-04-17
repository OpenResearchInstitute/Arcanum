// transforms.rs — GS scale and GM operations (Step 5)
//
// GS: uniform scale applied to all wire coordinates (not wire radii).
// GM: rotation (ROX → ROY → ROZ) then translation, optionally replicated.
//     n_copies == 0 → transform tagged segments in place.
//     n_copies > 0  → keep originals, append N transformed copies.
//
// Rotation convention: Rx(rox) → Ry(roy) → Rz(roz), angles in degrees.

use std::f64::consts::PI;

use arcanum_nec_import::{GeometryTransforms, WireDescription};
use nalgebra::{Matrix3, Vector3};

use crate::errors::GeometryError;
use crate::mesh::{
    CurveParams, Segment, TagMap,
};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn apply(
    segments: &mut Vec<Segment>,
    tag_map: &mut TagMap,
    transforms: &GeometryTransforms,
    _wires: &[WireDescription],
) -> Result<(), GeometryError> {
    // Step 5a: GS scale (applied before GM).
    if let Some(scale) = transforms.gs_scale {
        for seg in segments.iter_mut() {
            scale_segment(seg, scale);
        }
    }

    // Step 5b: GM operations, in deck order.
    for op in &transforms.gm_ops {
        let rot = rotation_matrix(op.rot_x, op.rot_y, op.rot_z);
        let trans = Vector3::new(op.trans_x, op.trans_y, op.trans_z);

        if op.n_copies == 0 {
            // Transform tagged segments in place.
            for seg in segments.iter_mut() {
                if op.tag == 0 || seg.tag == op.tag {
                    transform_segment(seg, &rot, &trans);
                }
            }
        } else {
            // Generate n_copies new copies; keep originals unchanged.
            let source_segments: Vec<Segment> = segments
                .iter()
                .filter(|s| op.tag == 0 || s.tag == op.tag)
                .cloned()
                .collect();

            for copy_k in 1..=(op.n_copies as usize) {
                // Each copy k applies the base transform k times cumulatively.
                let rot_k = rotation_matrix(
                    op.rot_x * copy_k as f64,
                    op.rot_y * copy_k as f64,
                    op.rot_z * copy_k as f64,
                );
                let trans_k = trans * copy_k as f64;
                let new_tag_offset = copy_k as u32 * op.tag_increment;

                let base_index = segments.len();
                for (local_k, src) in source_segments.iter().enumerate() {
                    let mut new_seg = src.clone();
                    new_seg.tag = src.tag + new_tag_offset;
                    new_seg.segment_index = base_index + local_k;
                    new_seg.wire_index = base_index + local_k; // placeholder
                    transform_segment(&mut new_seg, &rot_k, &trans_k);
                    segments.push(new_seg);
                }

                // Register the new copy in the tag map.
                if !source_segments.is_empty() {
                    let first = base_index;
                    let last = base_index + source_segments.len() - 1;
                    let new_tag = source_segments[0].tag + new_tag_offset;
                    tag_map.insert(new_tag, first, last);
                }
            }
        }
    }

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// GS: uniform coordinate scale (wire_radius is NOT scaled)
// ─────────────────────────────────────────────────────────────────────────────

fn scale_segment(seg: &mut Segment, s: f64) {
    match &mut seg.curve {
        CurveParams::Linear(p) => {
            p.start *= s;
            p.end *= s;
        }
        CurveParams::Arc(p) => {
            p.start *= s;
            p.end *= s;
            p.radius *= s;
        }
        CurveParams::Helix(p) => {
            p.start *= s;
            p.end *= s;
            p.total_length *= s;
            p.radius_start *= s;
            p.radius_end *= s;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GM: rotation then translation applied to Cartesian endpoints
// ─────────────────────────────────────────────────────────────────────────────

fn transform_segment(seg: &mut Segment, rot: &Matrix3<f64>, trans: &Vector3<f64>) {
    match &mut seg.curve {
        CurveParams::Linear(p) => {
            p.start = rot * p.start + trans;
            p.end = rot * p.end + trans;
        }
        CurveParams::Arc(p) => {
            // After GM the arc is in a rotated plane. theta1/theta2 are now
            // approximate; Phase 2 uses the Cartesian endpoints.
            p.start = rot * p.start + trans;
            p.end = rot * p.end + trans;
        }
        CurveParams::Helix(p) => {
            // After GM the helix is in a rotated frame.
            // Phase 2 uses the precomputed Cartesian endpoints.
            p.start = rot * p.start + trans;
            p.end = rot * p.end + trans;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Rotation matrix: Rz(roz) * Ry(roy) * Rx(rox), angles in degrees.
// NEC-2 convention: rotations applied in order ROX → ROY → ROZ.
// ─────────────────────────────────────────────────────────────────────────────

fn rotation_matrix(rox_deg: f64, roy_deg: f64, roz_deg: f64) -> Matrix3<f64> {
    let rx = deg_to_rad(rox_deg);
    let ry = deg_to_rad(roy_deg);
    let rz = deg_to_rad(roz_deg);

    let r_x = Matrix3::new(
        1.0,     0.0,      0.0,
        0.0,  rx.cos(), -rx.sin(),
        0.0,  rx.sin(),  rx.cos(),
    );
    let r_y = Matrix3::new(
        ry.cos(), 0.0, ry.sin(),
        0.0,      1.0, 0.0,
       -ry.sin(), 0.0, ry.cos(),
    );
    let r_z = Matrix3::new(
        rz.cos(), -rz.sin(), 0.0,
        rz.sin(),  rz.cos(), 0.0,
        0.0,       0.0,      1.0,
    );

    r_z * r_y * r_x
}

#[inline]
fn deg_to_rad(deg: f64) -> f64 {
    deg * PI / 180.0
}
