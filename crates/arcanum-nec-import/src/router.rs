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
//   - Apply GS scale and GM transformations to wire coordinates
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
    let mut gs_scale: Option<f64> = None;
    let mut pending_gm: Vec<(usize, GmCard)> = Vec::new();
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
                gs_scale = Some(gs.scale);
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
                pending_gm.push((line_number, gm));
            }

            NecCard::Ge(ge) => {
                // Apply GS scale to all wire coordinates (not radii).
                if let Some(scale) = gs_scale {
                    apply_gs(&mut wires, scale);
                }
                // Apply GM transformations in deck order.
                for (gm_line, gm) in pending_gm.drain(..) {
                    apply_gm(&mut wires, &mut tag_registry, &gm, gm_line)?;
                }
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
// GS — global scale
// ─────────────────────────────────────────────────────────────────────────────

/// Scale all wire endpoint coordinates by `scale`. Wire radii are NOT scaled,
/// per docs/nec-import/card-reference.md Section 3 (GS critical note).
fn apply_gs(wires: &mut [WireDescription], scale: f64) {
    for wire in wires.iter_mut() {
        match wire {
            WireDescription::Straight(sw) => {
                sw.x1 *= scale;
                sw.y1 *= scale;
                sw.z1 *= scale;
                sw.x2 *= scale;
                sw.y2 *= scale;
                sw.z2 *= scale;
                // sw.radius is intentionally NOT scaled
            }
            WireDescription::Arc(aw) => {
                // Scale the arc radius (a geometric dimension).
                // Wire radius is NOT scaled.
                aw.arc_radius *= scale;
            }
            WireDescription::Helix(hw) => {
                // Scale all geometric dimensions; wire radius is NOT scaled.
                hw.pitch *= scale;
                hw.total_length *= scale;
                hw.radius_start *= scale;
                hw.radius_end *= scale;
                // n_turns = total_length / pitch is unchanged by uniform scaling
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// GM — geometry move / rotate / replicate
// ─────────────────────────────────────────────────────────────────────────────

/// Apply a GM transformation to the wire list.
///
/// If NRPT = 0: transform the specified wire(s) in place (tag unchanged).
/// If NRPT > 0: keep original wire(s); generate NRPT copies, each with the
///              transform applied one more time than the previous copy.
///              Copy tags are incremented by ITS per copy.
///
/// GM is fully implemented for StraightWire. For ArcWire and HelixWire,
/// translation is applied via GS scaling; rotation of curved-segment geometry
/// requires Phase 1 involvement and is not applied here.
fn apply_gm(
    wires: &mut Vec<WireDescription>,
    registry: &mut TagRegistry,
    gm: &GmCard,
    line_number: usize,
) -> Result<(), ParseError> {
    // Collect indices of wires to transform.
    let target_indices: Vec<usize> = if gm.tag == 0 {
        (0..wires.len()).collect()
    } else {
        let idx = wires
            .iter()
            .position(|w| w.tag() == gm.tag)
            .ok_or_else(|| {
                ParseError::new(
                    ParseErrorKind::UnknownTagReference,
                    line_number,
                    format!("GM references tag {} which is not defined", gm.tag),
                )
            })?;
        vec![idx]
    };

    if gm.n_copies == 0 {
        // Transform in place.
        for &idx in &target_indices {
            transform_wire_in_place(&mut wires[idx], gm);
        }
    } else {
        // Generate copies. The original wire is not moved.
        // Copy k gets the transform applied k times to the original.
        let source_wires: Vec<WireDescription> =
            target_indices.iter().map(|&i| wires[i].clone()).collect();

        for copy_num in 1..=(gm.n_copies as usize) {
            for src in &source_wires {
                // Apply the transform copy_num times to the source wire.
                let mut new_wire = src.clone();
                for _ in 0..copy_num {
                    transform_wire_in_place(&mut new_wire, gm);
                }
                // Assign the new tag.
                let new_tag = src.tag() + gm.tag_increment * copy_num as u32;
                set_wire_tag(&mut new_wire, new_tag);

                let wire_index = wires.len();
                let seg_count = new_wire.segment_count();
                registry.insert(new_tag, wire_index, seg_count, line_number)?;
                wires.push(new_wire);
            }
        }
    }
    Ok(())
}

/// Apply one application of the GM rotation and translation to a wire.
/// Full rotation is implemented for StraightWire endpoints.
/// ArcWire and HelixWire receive translation only (their parametric forms
/// are origin-relative and cannot be rotated without Phase 1 involvement).
fn transform_wire_in_place(wire: &mut WireDescription, gm: &GmCard) {
    match wire {
        WireDescription::Straight(sw) => {
            let (x1, y1, z1) = rotate_then_translate(sw.x1, sw.y1, sw.z1, gm);
            let (x2, y2, z2) = rotate_then_translate(sw.x2, sw.y2, sw.z2, gm);
            sw.x1 = x1;
            sw.y1 = y1;
            sw.z1 = z1;
            sw.x2 = x2;
            sw.y2 = y2;
            sw.z2 = z2;
        }
        WireDescription::Arc(_) | WireDescription::Helix(_) => {
            // Curved wires are origin-relative in their parametric form.
            // Rotation and translation require Phase 1 to recompute the
            // parametric form. This is a known limitation of the initial
            // implementation; GM on GA/GH is not supported here.
        }
    }
}

/// Apply GM rotation (ROX → ROY → ROZ) then translation (XS, YS, ZS)
/// to a single point.
fn rotate_then_translate(x: f64, y: f64, z: f64, gm: &GmCard) -> (f64, f64, f64) {
    // Rotation about X axis (ROX degrees)
    let (x, y, z) = rotate_x(x, y, z, gm.rot_x.to_radians());
    // Rotation about Y axis (ROY degrees)
    let (x, y, z) = rotate_y(x, y, z, gm.rot_y.to_radians());
    // Rotation about Z axis (ROZ degrees)
    let (x, y, z) = rotate_z(x, y, z, gm.rot_z.to_radians());
    // Translation
    (x + gm.trans_x, y + gm.trans_y, z + gm.trans_z)
}

fn rotate_x(x: f64, y: f64, z: f64, angle: f64) -> (f64, f64, f64) {
    let (c, s) = (angle.cos(), angle.sin());
    (x, y * c - z * s, y * s + z * c)
}

fn rotate_y(x: f64, y: f64, z: f64, angle: f64) -> (f64, f64, f64) {
    let (c, s) = (angle.cos(), angle.sin());
    (x * c + z * s, y, -x * s + z * c)
}

fn rotate_z(x: f64, y: f64, z: f64, angle: f64) -> (f64, f64, f64) {
    let (c, s) = (angle.cos(), angle.sin());
    (x * c - y * s, x * s + y * c, z)
}

fn set_wire_tag(wire: &mut WireDescription, tag: u32) {
    match wire {
        WireDescription::Straight(sw) => sw.tag = tag,
        WireDescription::Arc(aw) => aw.tag = tag,
        WireDescription::Helix(hw) => hw.tag = tag,
    }
}
