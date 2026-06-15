// mesh.rs — Phase 1 output types
//
// The Mesh struct is the sole output of Phase 1 and the sole geometric
// input to all subsequent phases. It is immutable after Phase 1 completes.

use std::f64::consts::PI;

use nalgebra::{Matrix3, Vector3};

// ─────────────────────────────────────────────────────────────────────────────
// Curve representation
// ─────────────────────────────────────────────────────────────────────────────

/// The parametric curve type of a segment.
#[derive(Debug, Clone, PartialEq)]
pub enum CurveType {
    Linear,
    Arc,
    Helix,
}

/// Parametric description of a segment's geometry.
///
/// For each type, the segment is parameterized by σ ∈ [0, 1] where σ = 0 is
/// the segment start and σ = 1 is the segment end. Phase 2 evaluates r(σ),
/// r'(σ), and |r'(σ)| at Gauss-Legendre quadrature points.
#[derive(Debug, Clone)]
pub enum CurveParams {
    /// Straight wire segment.
    Linear(LinearParams),
    /// Circular arc segment in the XZ plane (or rotated via GM).
    Arc(ArcParams),
    /// Helical segment about the z-axis.
    Helix(HelixParams),
}

/// Parameters for a linear segment: r(σ) = start + σ(end − start).
#[derive(Debug, Clone)]
pub struct LinearParams {
    pub start: Vector3<f64>,
    pub end: Vector3<f64>,
}

impl LinearParams {
    /// Length of the segment.
    pub fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }
}

/// Parameters for an arc segment.
///
/// In the local frame, r(θ) = (R cos θ, 0, R sin θ) where θ ∈ [θ1, θ2].
/// The world-space position is: rotation * r_local(θ(σ)) + center.
/// Before any GM transform, rotation = I and center = 0.
#[derive(Debug, Clone)]
pub struct ArcParams {
    /// Arc radius (meters).
    pub radius: f64,
    /// Start angle of this segment (radians).
    pub theta1: f64,
    /// End angle of this segment (radians).
    pub theta2: f64,
    /// Rotation from local XZ-plane frame to world frame.
    pub rotation: Matrix3<f64>,
    /// Translation (world-space origin of the local frame).
    pub center: Vector3<f64>,
    /// Precomputed start point (for junction detection and image generation).
    pub start: Vector3<f64>,
    /// Precomputed end point.
    pub end: Vector3<f64>,
}

/// Parameters for a helical segment.
///
/// In the local frame, r(τ) = (A(τ) cos(2πNτ), A(τ) sin(2πNτ), HL·τ)
/// where τ = (segment_index + σ) / n_segments for σ ∈ [0,1].
/// The world-space position is: rotation * r_local(τ) + center.
#[derive(Debug, Clone)]
pub struct HelixParams {
    /// Radius at the start of the full helix (A₁).
    pub radius_start: f64,
    /// Radius at the end of the full helix (A₂).
    pub radius_end: f64,
    /// Total axial length of the full helix.
    pub total_length: f64,
    /// Total number of turns of the full helix.
    pub n_turns: f64,
    /// Total number of segments the full helix is divided into.
    pub n_segments: u32,
    /// Index of this segment within the full helix (0-based).
    pub segment_index: u32,
    /// Rotation from local frame to world frame.
    pub rotation: Matrix3<f64>,
    /// Translation (world-space origin of the local frame).
    pub center: Vector3<f64>,
    /// Precomputed start point.
    pub start: Vector3<f64>,
    /// Precomputed end point.
    pub end: Vector3<f64>,
}

impl CurveParams {
    /// Position r(σ) for σ ∈ [0, 1].
    pub fn evaluate(&self, sigma: f64) -> Vector3<f64> {
        match self {
            CurveParams::Linear(p) => p.start + sigma * (p.end - p.start),
            CurveParams::Arc(p) => {
                let theta = p.theta1 + sigma * (p.theta2 - p.theta1);
                let local = Vector3::new(p.radius * theta.cos(), 0.0, p.radius * theta.sin());
                p.rotation * local + p.center
            }
            CurveParams::Helix(p) => {
                let tau = (p.segment_index as f64 + sigma) / p.n_segments as f64;
                let a = p.radius_start + tau * (p.radius_end - p.radius_start);
                let angle = 2.0 * PI * p.n_turns * tau;
                let local = Vector3::new(a * angle.cos(), a * angle.sin(), p.total_length * tau);
                p.rotation * local + p.center
            }
        }
    }

    /// Tangent dr/dσ (unnormalized) at parameter σ ∈ [0, 1].
    pub fn tangent(&self, sigma: f64) -> Vector3<f64> {
        match self {
            CurveParams::Linear(p) => p.end - p.start,
            CurveParams::Arc(p) => {
                let theta = p.theta1 + sigma * (p.theta2 - p.theta1);
                let dtheta = p.theta2 - p.theta1;
                let local = Vector3::new(
                    -p.radius * theta.sin() * dtheta,
                    0.0,
                    p.radius * theta.cos() * dtheta,
                );
                p.rotation * local
            }
            CurveParams::Helix(p) => {
                let n_seg = p.n_segments as f64;
                let dtau = 1.0 / n_seg;
                let tau = (p.segment_index as f64 + sigma) / n_seg;
                let da = p.radius_end - p.radius_start;
                let a = p.radius_start + tau * da;
                let angle = 2.0 * PI * p.n_turns * tau;
                let dangle = 2.0 * PI * p.n_turns * dtau;
                let local = Vector3::new(
                    da * dtau * angle.cos() - a * angle.sin() * dangle,
                    da * dtau * angle.sin() + a * angle.cos() * dangle,
                    p.total_length * dtau,
                );
                p.rotation * local
            }
        }
    }

    /// Speed |dr/dσ| at parameter σ.
    pub fn speed(&self, sigma: f64) -> f64 {
        self.tangent(sigma).norm()
    }

    /// Total arc length of the segment.
    pub fn arc_length(&self) -> f64 {
        match self {
            CurveParams::Linear(p) => (p.end - p.start).norm(),
            CurveParams::Arc(p) => p.radius * (p.theta2 - p.theta1).abs(),
            CurveParams::Helix(_) => {
                // No closed-form for variable-radius helix; use 16-point GL.
                let n = 16;
                let mut sum = 0.0;
                for i in 0..n {
                    let sigma = (i as f64 + 0.5) / n as f64;
                    sum += self.speed(sigma);
                }
                sum / n as f64
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Material
// ─────────────────────────────────────────────────────────────────────────────

/// Wire material model.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum Material {
    /// Perfect electric conductor (default).
    #[default]
    PEC,
    /// Finite conductivity wire.
    Lossy { conductivity: f64 },
}

// ─────────────────────────────────────────────────────────────────────────────
// Segment
// ─────────────────────────────────────────────────────────────────────────────

/// A single discretized wire segment.
///
/// Segments are the fundamental unit of the mesh. Phase 2 integrates over
/// each segment to fill the impedance matrix.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Parametric curve description for Phase 2 integration.
    pub curve: CurveParams,
    /// Wire cross-section radius (meters). Not scaled by GS.
    pub wire_radius: f64,
    /// Material model.
    pub material: Material,
    /// NEC wire tag number. Image segments use the tag of their source wire.
    pub tag: u32,
    /// Global index of this segment in the mesh segment list.
    pub segment_index: usize,
    /// Index of the wire (GW/GA/GH card) this segment belongs to.
    pub wire_index: usize,
    /// True if this is a PEC ground image segment (not addressable by EX/LD).
    pub is_image: bool,
}

impl Segment {
    /// Precomputed start point of the segment.
    pub fn start(&self) -> Vector3<f64> {
        match &self.curve {
            CurveParams::Linear(p) => p.start,
            CurveParams::Arc(p) => p.start,
            CurveParams::Helix(p) => p.start,
        }
    }

    /// Precomputed end point of the segment.
    pub fn end(&self) -> Vector3<f64> {
        match &self.curve {
            CurveParams::Linear(p) => p.end,
            CurveParams::Arc(p) => p.end,
            CurveParams::Helix(p) => p.end,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Junction
// ─────────────────────────────────────────────────────────────────────────────

/// Which end of a segment is at a junction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EndpointSide {
    Start,
    End,
}

/// A reference to one endpoint of one segment.
#[derive(Debug, Clone)]
pub struct SegmentEndpoint {
    pub segment_index: usize,
    pub side: EndpointSide,
}

/// A point in space where two or more segment endpoints meet.
#[derive(Debug, Clone)]
pub struct Junction {
    /// Unique index of this junction.
    pub junction_index: usize,
    /// All segment endpoints at this junction.
    pub endpoints: Vec<SegmentEndpoint>,
    /// True if this junction is a self-loop (start and end of the same wire).
    pub is_self_loop: bool,
}

// ─────────────────────────────────────────────────────────────────────────────
// Ground descriptor
// ─────────────────────────────────────────────────────────────────────────────

/// Ground plane model passed through to Phase 2.
#[derive(Debug, Clone)]
pub struct GroundDescriptor {
    pub ground_type: GroundType,
    /// σ (S/m). None for PEC or free space.
    pub conductivity: Option<f64>,
    /// εᵣ (relative). None for PEC or free space.
    pub permittivity: Option<f64>,
    /// True if Phase 1 added image segments to the mesh.
    pub images_generated: bool,
}

/// Ground type, derived from the GN card IPERF field.
#[derive(Debug, Clone, PartialEq)]
pub enum GroundType {
    /// No GN card. Free space.
    None,
    /// IPERF = 0 or 2. Lossy ground; no image segments.
    Lossy,
    /// IPERF = 1. PEC; image segments generated by Phase 1.
    PEC,
}

impl Default for GroundDescriptor {
    fn default() -> Self {
        GroundDescriptor {
            ground_type: GroundType::None,
            conductivity: None,
            permittivity: None,
            images_generated: false,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Tag map
// ─────────────────────────────────────────────────────────────────────────────

/// Maps NEC wire tag numbers to segment index ranges in the mesh.
///
/// Image segments are excluded — they are not addressable by EX/LD cards.
#[derive(Debug, Clone, Default)]
pub struct TagMap {
    entries: Vec<TagEntry>,
}

#[derive(Debug, Clone)]
struct TagEntry {
    tag: u32,
    /// First segment index for this wire (inclusive).
    first: usize,
    /// Last segment index for this wire (inclusive).
    last: usize,
}

impl TagMap {
    pub fn new() -> Self {
        TagMap::default()
    }

    /// Register a wire's segment range.
    pub fn insert(&mut self, tag: u32, first: usize, last: usize) {
        self.entries.push(TagEntry { tag, first, last });
    }

    /// Look up the segment index range for a tag. Returns None if not found.
    pub fn get(&self, tag: u32) -> Option<(usize, usize)> {
        self.entries
            .iter()
            .find(|e| e.tag == tag)
            .map(|e| (e.first, e.last))
    }

    /// Number of segments for a given tag. Returns None if not found.
    pub fn segment_count(&self, tag: u32) -> Option<usize> {
        self.get(tag).map(|(first, last)| last - first + 1)
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, usize, usize)> + '_ {
        self.entries.iter().map(|e| (e.tag, e.first, e.last))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mesh — the Phase 1 output
// ─────────────────────────────────────────────────────────────────────────────

/// The complete discretized segment mesh produced by Phase 1.
///
/// Immutable after construction. All downstream phases consume this struct.
#[derive(Debug)]
pub struct Mesh {
    /// All segments in order: real segments first, image segments last.
    pub segments: Vec<Segment>,
    /// All detected junctions.
    pub junctions: Vec<Junction>,
    /// Bidirectional endpoint → junction index map.
    /// Indexed as `endpoint_junction[segment_index * 2 + side]`
    /// where side 0 = Start, side 1 = End. `None` means free endpoint.
    pub endpoint_junction: Vec<Option<usize>>,
    /// Ground plane descriptor.
    pub ground: GroundDescriptor,
    /// Tag → segment index range (real segments only).
    pub tag_map: TagMap,
}

impl Mesh {
    /// Number of real (non-image) segments.
    pub fn real_segment_count(&self) -> usize {
        self.segments.iter().filter(|s| !s.is_image).count()
    }

    /// Number of image segments.
    pub fn image_segment_count(&self) -> usize {
        self.segments.iter().filter(|s| s.is_image).count()
    }

    /// Look up which junction (if any) a segment endpoint belongs to.
    pub fn junction_at(&self, segment_index: usize, side: &EndpointSide) -> Option<usize> {
        let idx = segment_index * 2 + if *side == EndpointSide::Start { 0 } else { 1 };
        self.endpoint_junction.get(idx).copied().flatten()
    }
}
