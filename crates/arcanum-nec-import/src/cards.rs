// cards.rs
//
// Data types for the NEC import pipeline.
//
// Two layers of types are defined here:
//
//   1. Card structs and NecCard enum — the output of the lexical stage.
//      Each parsed line becomes one NecCard variant. These types are
//      pub(crate): they are an implementation detail of the parser and
//      do not cross the crate boundary.
//
//   2. SimulationInput and its fields — the output of the semantic stage.
//      These types are pub: they are consumed by arcanum-py and by each
//      computational phase crate.
//
// Field names and semantics match docs/nec-import/card-reference.md exactly.

// ─────────────────────────────────────────────────────────────────────────────
// 1. Card structs (pub(crate) — lexical stage output)
// ─────────────────────────────────────────────────────────────────────────────

/// GW — Straight wire between two endpoints.
#[derive(Debug, Clone)]
pub(crate) struct GwCard {
    pub tag: u32,
    pub segment_count: u32,
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
    pub x2: f64,
    pub y2: f64,
    pub z2: f64,
    pub radius: f64,
}

/// GA — Circular arc in the XZ plane.
#[derive(Debug, Clone)]
pub(crate) struct GaCard {
    pub tag: u32,
    pub segment_count: u32,
    pub arc_radius: f64,
    pub angle1: f64,
    pub angle2: f64,
    pub radius: f64,
}

/// GH — Helix along the z-axis.
#[derive(Debug, Clone)]
pub(crate) struct GhCard {
    pub tag: u32,
    pub segment_count: u32,
    pub pitch: f64,
    pub total_length: f64,
    pub radius_start: f64,
    pub radius_end: f64,
    pub radius: f64,
}

/// GM — Geometry move / rotate / replicate.
#[derive(Debug, Clone)]
pub(crate) struct GmCard {
    /// Tag of wire to transform. 0 = all wires.
    pub tag: u32,
    /// Number of additional copies to generate. 0 = transform in place.
    pub n_copies: u32,
    pub rot_x: f64,
    pub rot_y: f64,
    pub rot_z: f64,
    pub trans_x: f64,
    pub trans_y: f64,
    pub trans_z: f64,
    /// Tag increment applied to each generated copy.
    pub tag_increment: u32,
}

/// GS — Global coordinate scale factor.
#[derive(Debug, Clone)]
pub(crate) struct GsCard {
    pub scale: f64,
}

/// GE — Geometry end. Required deck terminator for the geometry section.
#[derive(Debug, Clone)]
pub(crate) struct GeCard {
    pub gpflag: i32,
}

/// GN — Ground definition.
#[derive(Debug, Clone)]
pub(crate) struct GnCard {
    /// Ground type: -1=free space, 0=lossy/reflection, 1=PEC, 2=Sommerfeld
    pub iperf: i32,
    /// Number of radial wires in ground screen. 0 if no screen.
    pub nradl: i32,
    /// Relative permittivity (used when iperf = 0 or 2).
    pub epse: f64,
    /// Conductivity in S/m (used when iperf = 0 or 2).
    pub sig: f64,
}

/// EX — Excitation source applied to a specific segment.
#[derive(Debug, Clone)]
pub(crate) struct ExCard {
    /// Excitation type. Supported: 0 (delta-gap voltage), 5 (current slope).
    pub ex_type: i32,
    pub tag: u32,
    pub segment: u32,
    pub voltage_real: f64,
    pub voltage_imag: f64,
}

/// LD — Impedance load on one or more segments.
#[derive(Debug, Clone)]
pub(crate) struct LdCard {
    /// Load type: 0=series RLC, 1=parallel RLC, 4=distributed RLC, 5=conductivity
    pub ld_type: i32,
    /// Tag of wire to load. 0 = all wires.
    pub tag: u32,
    /// First segment to load (1-indexed).
    pub first_seg: u32,
    /// Last segment to load (1-indexed). 0 = load only first_seg.
    pub last_seg: u32,
    pub zlr: f64,
    pub zli: f64,
    pub zlc: f64,
}

/// FR — Frequency or frequency sweep.
#[derive(Debug, Clone)]
pub(crate) struct FrCard {
    /// Stepping type: 0 = linear, 1 = multiplicative.
    pub ifrq: i32,
    pub nfrq: u32,
    /// Starting frequency in MHz.
    pub fmhz: f64,
    /// Frequency increment in MHz (linear) or multiplicative factor.
    pub delfrq: f64,
}

/// RP — Radiation pattern output request.
#[derive(Debug, Clone)]
pub(crate) struct RpCard {
    pub calc: i32,
    pub n_theta: u32,
    pub n_phi: u32,
    pub xnda: i32,
    pub theta_start: f64,
    pub phi_start: f64,
    pub d_theta: f64,
    pub d_phi: f64,
    /// Radial distance. 0.0 = far field.
    pub rfld: f64,
}

/// NE / NH — Near electric or magnetic field output request.
/// Both cards have identical field layouts.
#[derive(Debug, Clone)]
pub(crate) struct NearFieldCard {
    pub nx: u32,
    pub ny: u32,
    pub nz: u32,
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

// ─────────────────────────────────────────────────────────────────────────────
// NecCard enum — one variant per card type
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub(crate) enum NecCard {
    Gw(GwCard),
    Ga(GaCard),
    Gh(GhCard),
    Gm(GmCard),
    Gs(GsCard),
    Ge(GeCard),
    Gn(GnCard),
    Ex(ExCard),
    Ld(LdCard),
    Fr(FrCard),
    Rp(RpCard),
    Ne(NearFieldCard),
    Nh(NearFieldCard),
    En,
    /// Unrecognized mnemonic — preserved for warning emission.
    Unknown(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// ParsedDeck — output of the lexical stage, input to the semantic stage
// ─────────────────────────────────────────────────────────────────────────────

/// The ordered list of parsed cards produced by the lexical stage.
/// Line numbers are preserved alongside each card for error reporting.
#[derive(Debug)]
pub(crate) struct ParsedDeck {
    /// (line_number, card) pairs in deck order.
    pub cards: Vec<(usize, NecCard)>,
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. SimulationInput and fields (pub — semantic stage output)
// ─────────────────────────────────────────────────────────────────────────────

/// The fully parsed and routed simulation input.
/// Each computational phase receives only its own field from this struct.
#[derive(Debug, Clone)]
pub struct SimulationInput {
    /// Wire geometry and ground boundary condition → Phase 1.
    pub mesh_input: MeshInput,
    /// Frequency list in Hz → Phase 2, Phase 3.
    pub frequencies: Vec<f64>,
    /// Voltage/current sources → Phase 3.
    pub sources: Vec<SourceDefinition>,
    /// Impedance loads → Phase 3.
    pub loads: Vec<LoadDefinition>,
    /// Ground electrical parameters → Phase 2. None if free space or PEC.
    pub ground_electrical: Option<GroundElectrical>,
    /// Pattern and near-field output requests → Phase 4.
    pub output_requests: OutputRequests,
}

/// Wire geometry and ground boundary condition consumed by Phase 1.
#[derive(Debug, Clone, Default)]
pub struct MeshInput {
    /// Wire descriptions after GS and GM transformations have been applied.
    pub wires: Vec<WireDescription>,
    /// Ground plane boundary condition (geometric only).
    pub ground: GeometricGround,
    /// Ground plane flag from GE card.
    pub gpflag: i32,
}

/// A single wire element — straight, arc, or helix.
/// Coordinates reflect any GS/GM transformations applied by the router.
#[derive(Debug, Clone)]
pub enum WireDescription {
    Straight(StraightWire),
    Arc(ArcWire),
    Helix(HelixWire),
}

impl WireDescription {
    pub fn tag(&self) -> u32 {
        match self {
            WireDescription::Straight(w) => w.tag,
            WireDescription::Arc(w) => w.tag,
            WireDescription::Helix(w) => w.tag,
        }
    }

    pub fn segment_count(&self) -> u32 {
        match self {
            WireDescription::Straight(w) => w.segment_count,
            WireDescription::Arc(w) => w.segment_count,
            WireDescription::Helix(w) => w.segment_count,
        }
    }
}

/// Straight wire between two endpoints.
#[derive(Debug, Clone)]
pub struct StraightWire {
    pub tag: u32,
    pub segment_count: u32,
    pub x1: f64,
    pub y1: f64,
    pub z1: f64,
    pub x2: f64,
    pub y2: f64,
    pub z2: f64,
    pub radius: f64,
}

/// Circular arc in the XZ plane.
#[derive(Debug, Clone)]
pub struct ArcWire {
    pub tag: u32,
    pub segment_count: u32,
    pub arc_radius: f64,
    pub angle1: f64,
    pub angle2: f64,
    pub radius: f64,
}

/// Helix along the z-axis.
#[derive(Debug, Clone)]
pub struct HelixWire {
    pub tag: u32,
    pub segment_count: u32,
    pub pitch: f64,
    pub total_length: f64,
    pub radius_start: f64,
    pub radius_end: f64,
    pub radius: f64,
    /// Derived: total_length / pitch. Computed by the router and stored here
    /// so downstream phases and Python tests can read it without recomputing.
    pub n_turns: f64,
}

/// Ground plane boundary condition consumed by Phase 1.
#[derive(Debug, Clone)]
pub struct GeometricGround {
    pub ground_type: GroundType,
}

impl Default for GeometricGround {
    fn default() -> Self {
        GeometricGround {
            ground_type: GroundType::FreeSpace,
        }
    }
}

/// Ground plane type, derived from GN IPERF field.
#[derive(Debug, Clone, PartialEq)]
pub enum GroundType {
    /// IPERF = -1 or no GN card. No ground plane.
    FreeSpace,
    /// IPERF = 0. Finite ground, reflection coefficient approximation.
    Lossy,
    /// IPERF = 1. Perfect electric conductor. Phase 1 generates image segments.
    PEC,
    /// IPERF = 2. Finite ground, Sommerfeld/Wait theory.
    Sommerfeld,
}

impl GroundType {
    /// String representation used by the Python interface.
    pub fn as_str(&self) -> &'static str {
        match self {
            GroundType::FreeSpace => "FreeSpace",
            GroundType::Lossy => "Lossy",
            GroundType::PEC => "PEC",
            GroundType::Sommerfeld => "Sommerfeld",
        }
    }
}

/// Ground electrical parameters consumed by Phase 2.
/// Present only when IPERF = 0 or 2.
#[derive(Debug, Clone)]
pub struct GroundElectrical {
    pub permittivity: f64,
    pub conductivity: f64,
    pub model: GroundModel,
}

/// Ground model variant for Phase 2 computation.
#[derive(Debug, Clone, PartialEq)]
pub enum GroundModel {
    /// IPERF = 0. Reflection coefficient approximation.
    ReflectionCoeff,
    /// IPERF = 2. Sommerfeld/Wait integral method.
    Sommerfeld,
}

/// A voltage or current source applied to a specific segment.
#[derive(Debug, Clone)]
pub struct SourceDefinition {
    /// Excitation type: 0 = delta-gap voltage, 5 = current slope discontinuity.
    pub ex_type: i32,
    pub tag: u32,
    pub segment: u32,
    pub voltage_real: f64,
    pub voltage_imag: f64,
}

/// An impedance load applied to one or more segments.
#[derive(Debug, Clone)]
pub struct LoadDefinition {
    /// Load type: 0=series RLC, 1=parallel RLC, 4=distributed, 5=conductivity.
    pub ld_type: i32,
    /// Tag of wire to load. 0 = all wires.
    pub tag: u32,
    pub first_seg: u32,
    pub last_seg: u32,
    pub zlr: f64,
    pub zli: f64,
    pub zlc: f64,
}

/// All output requests for Phase 4.
#[derive(Debug, Clone, Default)]
pub struct OutputRequests {
    pub radiation_patterns: Vec<RadiationPatternRequest>,
    pub near_e_fields: Vec<NearFieldRequest>,
    pub near_h_fields: Vec<NearFieldRequest>,
}

/// Far-field radiation pattern computation request.
#[derive(Debug, Clone)]
pub struct RadiationPatternRequest {
    pub calc: i32,
    pub n_theta: u32,
    pub n_phi: u32,
    pub xnda: i32,
    pub theta_start: f64,
    pub phi_start: f64,
    pub d_theta: f64,
    pub d_phi: f64,
    /// Radial distance. 0.0 = far field.
    pub rfld: f64,
}

/// Near electric or magnetic field computation request on a rectangular grid.
#[derive(Debug, Clone)]
pub struct NearFieldRequest {
    pub nx: u32,
    pub ny: u32,
    pub nz: u32,
    pub xo: f64,
    pub yo: f64,
    pub zo: f64,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}
