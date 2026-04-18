// discretize.rs — Wire discretization into segments (Steps 2–4)

use std::f64::consts::PI;

use arcanum_nec_import::{ArcWire, HelixWire, StraightWire, WireDescription};
use nalgebra::Vector3;

use crate::errors::{GeometryError, GeometryErrorKind, GeometryWarnings};
use crate::mesh::{ArcParams, CurveParams, HelixParams, LinearParams, Material, Segment, TagMap};

pub(crate) fn discretize_wires(
    wires: &[WireDescription],
    _warnings: &mut GeometryWarnings,
) -> Result<(Vec<Segment>, TagMap), GeometryError> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut tag_map = TagMap::new();

    for (wire_index, wire) in wires.iter().enumerate() {
        let first = segments.len();
        match wire {
            WireDescription::Straight(w) => discretize_straight(w, wire_index, &mut segments)?,
            WireDescription::Arc(w) => discretize_arc(w, wire_index, &mut segments)?,
            WireDescription::Helix(w) => discretize_helix(w, wire_index, &mut segments)?,
        }
        let last = segments.len() - 1;
        tag_map.insert(wire.tag(), first, last);
    }

    Ok((segments, tag_map))
}

// ─────────────────────────────────────────────────────────────────────────────
// Step 2 — Linear (GW)
// ─────────────────────────────────────────────────────────────────────────────

fn discretize_straight(
    wire: &StraightWire,
    wire_index: usize,
    segments: &mut Vec<Segment>,
) -> Result<(), GeometryError> {
    let n = wire.segment_count as usize;

    if n == 0 {
        return Err(GeometryError::new(
            GeometryErrorKind::ZeroSegmentCount,
            wire_index,
            format!("GW tag={} has segment count 0", wire.tag),
        ));
    }

    let r_a = Vector3::new(wire.x1, wire.y1, wire.z1);
    let r_b = Vector3::new(wire.x2, wire.y2, wire.z2);

    if (r_b - r_a).norm() == 0.0 {
        return Err(GeometryError::new(
            GeometryErrorKind::ZeroLengthWire,
            wire_index,
            format!(
                "GW tag={} has identical start and end coordinates",
                wire.tag
            ),
        ));
    }

    let base_index = segments.len();
    for k in 0..n {
        // Evaluate endpoints from closed-form — never accumulate incrementally.
        let t0 = k as f64 / n as f64;
        let t1 = (k + 1) as f64 / n as f64;
        let start = r_a + t0 * (r_b - r_a);
        let end = r_a + t1 * (r_b - r_a);

        segments.push(Segment {
            curve: CurveParams::Linear(LinearParams { start, end }),
            wire_radius: wire.radius,
            material: Material::PEC,
            tag: wire.tag,
            segment_index: base_index + k,
            wire_index,
            is_image: false,
        });
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Step 3 — Arc (GA)
// ─────────────────────────────────────────────────────────────────────────────

fn discretize_arc(
    wire: &ArcWire,
    wire_index: usize,
    segments: &mut Vec<Segment>,
) -> Result<(), GeometryError> {
    let n = wire.segment_count as usize;
    let theta1 = wire.angle1.to_radians();
    let theta2 = wire.angle2.to_radians();
    let r = wire.arc_radius;

    // Evaluate arc point in XZ plane: r(θ) = (R cosθ, 0, R sinθ)
    let arc_point =
        |theta: f64| -> Vector3<f64> { Vector3::new(r * theta.cos(), 0.0, r * theta.sin()) };

    let base_index = segments.len();
    for k in 0..n {
        // Closed-form angle bounds for segment k.
        let t0 = k as f64 / n as f64;
        let t1 = (k + 1) as f64 / n as f64;
        let th1k = theta1 + t0 * (theta2 - theta1);
        let th2k = theta1 + t1 * (theta2 - theta1);

        let start = arc_point(th1k);
        let end = arc_point(th2k);

        segments.push(Segment {
            curve: CurveParams::Arc(ArcParams {
                radius: r,
                theta1: th1k,
                theta2: th2k,
                start,
                end,
            }),
            wire_radius: wire.radius,
            material: Material::PEC,
            tag: wire.tag,
            segment_index: base_index + k,
            wire_index,
            is_image: false,
        });
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Step 4 — Helix (GH)
// ─────────────────────────────────────────────────────────────────────────────

fn discretize_helix(
    wire: &HelixWire,
    wire_index: usize,
    segments: &mut Vec<Segment>,
) -> Result<(), GeometryError> {
    let n = wire.segment_count as usize;
    let a1 = wire.radius_start;
    let a2 = wire.radius_end;
    let hl = wire.total_length;
    let n_turns = wire.n_turns;

    // Radius at parameter t ∈ [0,1] of the full helix.
    let radius_at = |t: f64| -> f64 { a1 + t * (a2 - a1) };

    // Helix position at parameter t ∈ [0,1]:
    // r(t) = (A(t) cos(2π N t), A(t) sin(2π N t), HL·t)
    let helix_point = |t: f64| -> Vector3<f64> {
        let a = radius_at(t);
        let angle = 2.0 * PI * n_turns * t;
        Vector3::new(a * angle.cos(), a * angle.sin(), hl * t)
    };

    let base_index = segments.len();
    for k in 0..n {
        // Closed-form parameter values — never accumulated.
        let t0 = k as f64 / n as f64;
        let t1 = (k + 1) as f64 / n as f64;
        let start = helix_point(t0);
        let end = helix_point(t1);

        segments.push(Segment {
            curve: CurveParams::Helix(HelixParams {
                radius_start: a1,
                radius_end: a2,
                total_length: hl,
                n_turns,
                n_segments: n as u32,
                segment_index: k as u32,
                start,
                end,
            }),
            wire_radius: wire.radius,
            material: Material::PEC,
            tag: wire.tag,
            segment_index: base_index + k,
            wire_index,
            is_image: false,
        });
    }
    Ok(())
}
