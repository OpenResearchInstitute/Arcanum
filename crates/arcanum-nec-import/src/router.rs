// router.rs
//
// Stage 2 — Semantic routing.
//
// Consumes a ParsedDeck from the lexical stage and produces a SimulationInput.
// Responsibilities:
//
//   - Enforce card ordering (geometry cards before GE; GE required)
//   - Validate field values for each card type
//   - Build the tag registry from GW/GA/GH cards; hard-error on duplicates
//   - Record GS and GM cards verbatim in GeometryTransforms (NOT applied here —
//     Phase 1 owns coordinate transformation per docs/phase1-geometry/design.md)
//   - Register GM-generated copy tags in the tag registry (for EX/LD validation)
//     without modifying wire coordinates
//   - Split the GN card: ground type → MeshInput; electrical params → ground_electrical
//   - Validate EX and LD tag/segment references against the complete tag registry
//   - Assemble the frequency list from FR cards (MHz → Hz conversion here)
//   - Emit warnings for unknown/unsupported cards, NRADL > 0, missing EN
//   - Assemble and return SimulationInput
//
// Note on GN placement: xnec2c places GN before GE; standard NEC2 places it
// after. Both are accepted. Only one GN card per deck is permitted.

use crate::cards::*;
use crate::errors::*;
use crate::tag_registry::TagRegistry;

/// Card mnemonics that are recognized but not implemented in the initial scope.
/// These produce UnsupportedCard warnings rather than UnknownCard warnings.
const KNOWN_UNSUPPORTED: &[&str] = &["GR", "SP", "SM", "NT", "TL", "KH", "NX", "PQ", "PT", "XQ"];

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

pub(crate) fn route(deck: ParsedDeck) -> Result<(SimulationInput, ParseWarnings), ParseError> {
    let mut warnings = ParseWarnings::new();
    let mut tag_registry = TagRegistry::new();

    // ── Geometry accumulation ─────────────────────────────────────────────
    let mut wires: Vec<WireDescription> = Vec::new();
    let mut transforms = GeometryTransforms::default();
    let mut ge_gpflag: i32 = 0;
    let mut ge_seen = false;

    // ── Deferred cross-reference validation ───────────────────────────────
    // EX and LD references are validated after the full tag registry is built.
    let mut pending_ex: Vec<(usize, ExCard)> = Vec::new();
    let mut pending_ld: Vec<(usize, LdCard)> = Vec::new();

    // ── Simulation parameters ─────────────────────────────────────────────
    let mut gn_card: Option<(usize, GnCard)> = None;
    let mut frequencies: Vec<f64> = Vec::new();
    let mut output_requests = OutputRequests::default();
    let mut en_seen = false;

    // ── First pass: dispatch each card ────────────────────────────────────
    for (line_number, card) in deck.cards {
        match card {
            // ── Geometry cards (must precede GE) ─────────────────────────
            NecCard::Gw(gw) => {
                if ge_seen {
                    return Err(geometry_after_ge("GW", line_number));
                }
                validate_gw(&gw, line_number)?;
                let idx = wires.len();
                tag_registry.insert(gw.tag, idx, gw.segment_count, line_number)?;
                wires.push(WireDescription::Straight(StraightWire {
                    tag: gw.tag,
                    segment_count: gw.segment_count,
                    x1: gw.x1,
                    y1: gw.y1,
                    z1: gw.z1,
                    x2: gw.x2,
                    y2: gw.y2,
                    z2: gw.z2,
                    radius: gw.radius,
                }));
            }

            NecCard::Ga(ga) => {
                if ge_seen {
                    return Err(geometry_after_ge("GA", line_number));
                }
                validate_ga(&ga, line_number)?;
                let idx = wires.len();
                tag_registry.insert(ga.tag, idx, ga.segment_count, line_number)?;
                wires.push(WireDescription::Arc(ArcWire {
                    tag: ga.tag,
                    segment_count: ga.segment_count,
                    arc_radius: ga.arc_radius,
                    angle1: ga.angle1,
                    angle2: ga.angle2,
                    radius: ga.radius,
                }));
            }

            NecCard::Gh(gh) => {
                if ge_seen {
                    return Err(geometry_after_ge("GH", line_number));
                }
                validate_gh(&gh, line_number)?;
                let n_turns = gh.total_length / gh.pitch;
                let idx = wires.len();
                tag_registry.insert(gh.tag, idx, gh.segment_count, line_number)?;
                wires.push(WireDescription::Helix(HelixWire {
                    tag: gh.tag,
                    segment_count: gh.segment_count,
                    pitch: gh.pitch,
                    total_length: gh.total_length,
                    radius_start: gh.radius_start,
                    radius_end: gh.radius_end,
                    radius: gh.radius,
                    n_turns,
                }));
            }

            NecCard::Gs(gs) => {
                if ge_seen {
                    return Err(geometry_after_ge("GS", line_number));
                }
                if gs.scale == 0.0 {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidFieldValue,
                        line_number,
                        "GS XSCALE must not be zero".to_string(),
                    ));
                }
                // Store for Phase 1; do not apply to coordinates here.
                transforms.gs_scale = Some(gs.scale);
            }

            NecCard::Gm(gm) => {
                if ge_seen {
                    return Err(geometry_after_ge("GM", line_number));
                }
                if gm.n_copies > 0 && gm.tag_increment == 0 {
                    return Err(ParseError::new(
                        ParseErrorKind::InvalidFieldValue,
                        line_number,
                        "GM ITS (tag_increment) must be > 0 when NRPT > 0 \
                         to avoid producing duplicate tags"
                            .to_string(),
                    ));
                }
                // Register copy tags in the tag registry so EX/LD validation
                // can resolve references to GM-generated wires. Coordinates are
                // not modified; Phase 1 will apply the transformation.
                if gm.n_copies > 0 {
                    register_gm_copies(&gm, &wires, &mut tag_registry, line_number)?;
                }
                transforms.gm_ops.push(GmOperation {
                    tag: gm.tag,
                    n_copies: gm.n_copies,
                    rot_x: gm.rot_x,
                    rot_y: gm.rot_y,
                    rot_z: gm.rot_z,
                    trans_x: gm.trans_x,
                    trans_y: gm.trans_y,
                    trans_z: gm.trans_z,
                    tag_increment: gm.tag_increment,
                });
            }

            NecCard::Ge(ge) => {
                ge_gpflag = ge.gpflag;
                ge_seen = true;
            }

            // ── GN: accepted before or after GE ──────────────────────────
            NecCard::Gn(gn) => {
                if gn_card.is_some() {
                    return Err(ParseError::new(
                        ParseErrorKind::MultipleGnCards,
                        line_number,
                        "only one GN card is permitted per deck".to_string(),
                    ));
                }
                if gn.nradl > 0 {
                    warnings.push(ParseWarning::new(
                        ParseWarningKind::NradlIgnored,
                        line_number,
                        format!(
                            "GN NRADL = {} (radial ground screen) is not supported \
                             in initial implementation; value ignored",
                            gn.nradl
                        ),
                    ));
                }
                validate_gn(&gn, line_number)?;
                gn_card = Some((line_number, gn));
            }

            // ── EX / LD: collect for deferred validation ──────────────────
            NecCard::Ex(ex) => match ex.ex_type {
                0 | 5 => pending_ex.push((line_number, ex)),
                t => warnings.push(ParseWarning::new(
                    ParseWarningKind::UnsupportedExType,
                    line_number,
                    format!(
                        "EX EXTYPE = {} is not supported in initial implementation; \
                             card skipped",
                        t
                    ),
                )),
            },

            NecCard::Ld(ld) => {
                pending_ld.push((line_number, ld));
            }

            // ── Simulation parameter cards ────────────────────────────────
            NecCard::Fr(fr) => {
                validate_fr(&fr, line_number)?;
                expand_frequencies(&fr, &mut frequencies);
            }

            NecCard::Rp(rp) => {
                output_requests
                    .radiation_patterns
                    .push(RadiationPatternRequest {
                        calc: rp.calc,
                        n_theta: rp.n_theta,
                        n_phi: rp.n_phi,
                        xnda: rp.xnda,
                        theta_start: rp.theta_start,
                        phi_start: rp.phi_start,
                        d_theta: rp.d_theta,
                        d_phi: rp.d_phi,
                        rfld: rp.rfld,
                    });
            }

            NecCard::Ne(ne) => {
                output_requests.near_e_fields.push(near_field_request(&ne));
            }

            NecCard::Nh(nh) => {
                output_requests.near_h_fields.push(near_field_request(&nh));
            }

            NecCard::En => {
                en_seen = true;
            }

            NecCard::Unknown(mnemonic) => {
                let kind = if KNOWN_UNSUPPORTED.contains(&mnemonic.as_str()) {
                    ParseWarningKind::UnsupportedCard
                } else {
                    ParseWarningKind::UnknownCard
                };
                warnings.push(ParseWarning::new(
                    kind,
                    line_number,
                    format!("card {:?} is not supported; skipped", mnemonic),
                ));
            }
        }
    }

    // ── Post-loop checks ──────────────────────────────────────────────────

    if !ge_seen {
        return Err(ParseError::new(
            ParseErrorKind::MissingGeCard,
            0,
            "deck does not contain a required GE card".to_string(),
        ));
    }

    if !en_seen {
        warnings.push(ParseWarning::new(
            ParseWarningKind::MissingEnCard,
            0,
            "deck does not contain an EN card; processing continues".to_string(),
        ));
    }

    // Validate EX references against the complete tag registry.
    let mut sources: Vec<SourceDefinition> = Vec::new();
    for (line_number, ex) in pending_ex {
        tag_registry.resolve(ex.tag, ex.segment, line_number)?;
        sources.push(SourceDefinition {
            ex_type: ex.ex_type,
            tag: ex.tag,
            segment: ex.segment,
            voltage_real: ex.voltage_real,
            voltage_imag: ex.voltage_imag,
        });
    }

    // Validate LD references against the complete tag registry.
    let mut loads: Vec<LoadDefinition> = Vec::new();
    for (line_number, ld) in pending_ld {
        validate_ld(&ld, &tag_registry, line_number)?;
        loads.push(LoadDefinition {
            ld_type: ld.ld_type,
            tag: ld.tag,
            first_seg: ld.first_seg,
            last_seg: ld.last_seg,
            zlr: ld.zlr,
            zli: ld.zli,
            zlc: ld.zlc,
        });
    }

    // Build GeometricGround and GroundElectrical from the GN card (if present).
    let (ground, ground_electrical) = match gn_card {
        Some((_, gn)) => build_ground(gn),
        None => (GeometricGround::default(), None),
    };

    Ok((
        SimulationInput {
            mesh_input: MeshInput {
                wires,
                ground,
                gpflag: ge_gpflag,
                transforms,
            },
            frequencies,
            sources,
            loads,
            ground_electrical,
            output_requests,
        },
        warnings,
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// Validation helpers
// ─────────────────────────────────────────────────────────────────────────────

fn geometry_after_ge(card: &str, line_number: usize) -> ParseError {
    ParseError::new(
        ParseErrorKind::GeometryAfterGe,
        line_number,
        format!(
            "{} card at line {} appears after GE; geometry cards must precede GE",
            card, line_number
        ),
    )
}

fn validate_gw(gw: &GwCard, line_number: usize) -> Result<(), ParseError> {
    if gw.tag == 0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            "GW ITAG must be > 0".to_string(),
        ));
    }
    if gw.segment_count == 0 {
        return Err(ParseError::new(
            ParseErrorKind::ZeroSegmentCount,
            line_number,
            "GW NS must be >= 1".to_string(),
        ));
    }
    let dx = gw.x2 - gw.x1;
    let dy = gw.y2 - gw.y1;
    let dz = gw.z2 - gw.z1;
    if dx == 0.0 && dy == 0.0 && dz == 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::ZeroLengthWire,
            line_number,
            "GW end1 and end2 are identical (zero-length wire)".to_string(),
        ));
    }
    if gw.radius <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GW RAD = {} must be > 0", gw.radius),
        ));
    }
    Ok(())
}

fn validate_ga(ga: &GaCard, line_number: usize) -> Result<(), ParseError> {
    if ga.tag == 0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            "GA ITAG must be > 0".to_string(),
        ));
    }
    if ga.segment_count == 0 {
        return Err(ParseError::new(
            ParseErrorKind::ZeroSegmentCount,
            line_number,
            "GA NS must be >= 1".to_string(),
        ));
    }
    if ga.arc_radius <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GA RADA = {} must be > 0", ga.arc_radius),
        ));
    }
    if (ga.angle2 - ga.angle1).abs() < f64::EPSILON {
        return Err(ParseError::new(
            ParseErrorKind::ZeroLengthWire,
            line_number,
            "GA ANG1 and ANG2 are equal (zero-length arc)".to_string(),
        ));
    }
    if ga.radius <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GA RAD = {} must be > 0", ga.radius),
        ));
    }
    Ok(())
}

fn validate_gh(gh: &GhCard, line_number: usize) -> Result<(), ParseError> {
    if gh.tag == 0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            "GH ITAG must be > 0".to_string(),
        ));
    }
    if gh.segment_count == 0 {
        return Err(ParseError::new(
            ParseErrorKind::ZeroSegmentCount,
            line_number,
            "GH NS must be >= 1".to_string(),
        ));
    }
    if gh.pitch == 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            "GH S (pitch) must not be zero".to_string(),
        ));
    }
    if gh.total_length <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GH HL = {} must be > 0", gh.total_length),
        ));
    }
    if gh.radius_start <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GH A1 = {} must be > 0", gh.radius_start),
        ));
    }
    if gh.radius_end <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GH A2 = {} must be > 0", gh.radius_end),
        ));
    }
    if gh.radius <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("GH RAD = {} must be > 0", gh.radius),
        ));
    }
    Ok(())
}

fn validate_gn(gn: &GnCard, line_number: usize) -> Result<(), ParseError> {
    match gn.iperf {
        -1..=2 => {}
        other => {
            return Err(ParseError::new(
                ParseErrorKind::InvalidFieldValue,
                line_number,
                format!("GN IPERF = {} is invalid; must be -1, 0, 1, or 2", other),
            ));
        }
    }
    if gn.iperf == 0 || gn.iperf == 2 {
        if gn.epse <= 0.0 {
            return Err(ParseError::new(
                ParseErrorKind::InvalidFieldValue,
                line_number,
                format!(
                    "GN EPSE = {} must be > 0 when IPERF = {}",
                    gn.epse, gn.iperf
                ),
            ));
        }
        if gn.sig < 0.0 {
            return Err(ParseError::new(
                ParseErrorKind::InvalidFieldValue,
                line_number,
                format!("GN SIG = {} must be >= 0", gn.sig),
            ));
        }
    }
    Ok(())
}

fn validate_fr(fr: &FrCard, line_number: usize) -> Result<(), ParseError> {
    if fr.nfrq < 1 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("FR NFRQ = {} must be >= 1", fr.nfrq),
        ));
    }
    if fr.fmhz <= 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            format!("FR FMHZ = {} must be > 0", fr.fmhz),
        ));
    }
    if fr.nfrq > 1 && fr.delfrq == 0.0 {
        return Err(ParseError::new(
            ParseErrorKind::InvalidFieldValue,
            line_number,
            "FR DELFRQ must not be zero when NFRQ > 1".to_string(),
        ));
    }
    Ok(())
}

fn validate_ld(ld: &LdCard, registry: &TagRegistry, line_number: usize) -> Result<(), ParseError> {
    // tag = 0 means load all wires; no per-tag validation needed.
    if ld.tag == 0 {
        return Ok(());
    }
    let entry = registry.get(ld.tag, line_number)?;
    if ld.first_seg == 0 || ld.first_seg > entry.segment_count {
        return Err(ParseError::new(
            ParseErrorKind::SegmentOutOfRange,
            line_number,
            format!(
                "LD LDTAGF = {} is out of range for tag {} (NS = {})",
                ld.first_seg, ld.tag, entry.segment_count
            ),
        ));
    }
    // last_seg = 0 means "load only first_seg"; no further range check needed.
    if ld.last_seg > 0 {
        if ld.last_seg < ld.first_seg {
            return Err(ParseError::new(
                ParseErrorKind::InvalidFieldValue,
                line_number,
                format!(
                    "LD LDTAGT = {} must be >= LDTAGF = {}",
                    ld.last_seg, ld.first_seg
                ),
            ));
        }
        if ld.last_seg > entry.segment_count {
            return Err(ParseError::new(
                ParseErrorKind::SegmentOutOfRange,
                line_number,
                format!(
                    "LD LDTAGT = {} is out of range for tag {} (NS = {})",
                    ld.last_seg, ld.tag, entry.segment_count
                ),
            ));
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Frequency expansion
// ─────────────────────────────────────────────────────────────────────────────

/// Expand an FR card into individual frequencies in Hz and append to the list.
fn expand_frequencies(fr: &FrCard, frequencies: &mut Vec<f64>) {
    let start_hz = fr.fmhz * 1.0e6;
    match fr.ifrq {
        0 => {
            // Linear stepping: f_n = fmhz + n * delfrq (MHz), converted to Hz.
            for i in 0..fr.nfrq {
                frequencies.push(start_hz + i as f64 * fr.delfrq * 1.0e6);
            }
        }
        1 => {
            // Multiplicative stepping: f_n = fmhz * delfrq^n.
            let mut f = start_hz;
            for _ in 0..fr.nfrq {
                frequencies.push(f);
                f *= fr.delfrq;
            }
        }
        _ => {
            // Unknown IFRQ: add only the start frequency.
            frequencies.push(start_hz);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Ground construction
// ─────────────────────────────────────────────────────────────────────────────

fn build_ground(gn: GnCard) -> (GeometricGround, Option<GroundElectrical>) {
    let ground_type = match gn.iperf {
        -1 => GroundType::FreeSpace,
        0 => GroundType::Lossy,
        1 => GroundType::PEC,
        2 => GroundType::Sommerfeld,
        _ => GroundType::FreeSpace, // already validated; unreachable in practice
    };

    let ground = GeometricGround { ground_type };

    let ground_electrical = match gn.iperf {
        0 => Some(GroundElectrical {
            permittivity: gn.epse,
            conductivity: gn.sig,
            model: GroundModel::ReflectionCoeff,
        }),
        2 => Some(GroundElectrical {
            permittivity: gn.epse,
            conductivity: gn.sig,
            model: GroundModel::Sommerfeld,
        }),
        _ => None,
    };

    (ground, ground_electrical)
}

// ─────────────────────────────────────────────────────────────────────────────
// Output request construction
// ─────────────────────────────────────────────────────────────────────────────

fn near_field_request(card: &NearFieldCard) -> NearFieldRequest {
    NearFieldRequest {
        nx: card.nx,
        ny: card.ny,
        nz: card.nz,
        xo: card.xo,
        yo: card.yo,
        zo: card.zo,
        dx: card.dx,
        dy: card.dy,
        dz: card.dz,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GM — tag registration for copy validation
// ─────────────────────────────────────────────────────────────────────────────

/// Register the tags that a GM card with NRPT > 0 will generate, so that EX
/// and LD cards referencing those tags can be validated at parse time.
///
/// Coordinates are NOT modified here. Phase 1 applies the actual transformation.
fn register_gm_copies(
    gm: &GmCard,
    wires: &[WireDescription],
    registry: &mut TagRegistry,
    line_number: usize,
) -> Result<(), ParseError> {
    let source_wires: Vec<(u32, u32)> = if gm.tag == 0 {
        wires.iter().map(|w| (w.tag(), w.segment_count())).collect()
    } else {
        let src = wires.iter().find(|w| w.tag() == gm.tag).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnknownTagReference,
                line_number,
                format!("GM references tag {} which is not defined", gm.tag),
            )
        })?;
        vec![(src.tag(), src.segment_count())]
    };

    for copy_num in 1..=(gm.n_copies as usize) {
        for &(src_tag, seg_count) in &source_wires {
            let new_tag = src_tag + gm.tag_increment * copy_num as u32;
            // Wire index is a placeholder — Phase 1 assigns real indices.
            registry.insert(new_tag, usize::MAX, seg_count, line_number)?;
        }
    }
    Ok(())
}
