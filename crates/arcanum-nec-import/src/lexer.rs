// lexer.rs
//
// Stage 1 — Lexical parsing.
//
// Converts raw .nec text into an ordered list of typed NecCard values
// (a ParsedDeck). This stage is responsible only for:
//
//   - Normalizing line endings (\r\n → \n)
//   - Skipping blank lines and CM/CE comment lines
//   - Extracting the 2-character card mnemonic from each line
//   - Tokenizing the remaining fields with split_ascii_whitespace()
//   - Parsing integer fields (i32) and float fields (f64)
//   - Hard-erroring on type mismatches and missing required fields
//   - Returning NecCard::Unknown for unrecognized mnemonics
//
// This stage does NOT validate card ordering, check tag references,
// or apply any transformations. Those are the router's responsibility.

use crate::cards::{
    ExCard, FrCard, GaCard, GeCard, GhCard, GmCard, GnCard, GsCard, GwCard, LdCard, NearFieldCard,
    NecCard, ParsedDeck, RpCard,
};
use crate::errors::{ParseError, ParseErrorKind};

// ─────────────────────────────────────────────────────────────────────────────
// Public entry point
// ─────────────────────────────────────────────────────────────────────────────

/// Parse raw .nec text into a ParsedDeck.
///
/// Returns a hard error on the first malformed field. Unknown card mnemonics
/// produce NecCard::Unknown entries, not errors.
pub(crate) fn lex(input: &str) -> Result<ParsedDeck, ParseError> {
    let normalized = input.replace("\r\n", "\n");
    let mut cards: Vec<(usize, NecCard)> = Vec::new();

    for (idx, raw_line) in normalized.lines().enumerate() {
        let line_number = idx + 1;
        let line = raw_line.trim();

        if line.is_empty() {
            continue;
        }

        // Mnemonic is the first two characters, uppercased.
        // Lines shorter than 2 chars cannot carry a valid mnemonic.
        if line.len() < 2 {
            continue;
        }
        let mnemonic = line[..2].to_ascii_uppercase();

        // CM and CE are comment cards — discard silently.
        if mnemonic == "CM" || mnemonic == "CE" {
            continue;
        }

        let card = parse_card(&mnemonic, line, line_number)?;
        cards.push((line_number, card));
    }

    Ok(ParsedDeck { cards })
}

// ─────────────────────────────────────────────────────────────────────────────
// Card dispatch
// ─────────────────────────────────────────────────────────────────────────────

fn parse_card(mnemonic: &str, line: &str, line_number: usize) -> Result<NecCard, ParseError> {
    let mut t = Tokens::new(line, line_number, mnemonic);

    match mnemonic {
        "GW" => parse_gw(&mut t),
        "GA" => parse_ga(&mut t),
        "GH" => parse_gh(&mut t),
        "GM" => parse_gm(&mut t),
        "GS" => parse_gs(&mut t),
        "GE" => parse_ge(&mut t),
        "GN" => parse_gn(&mut t),
        "EX" => parse_ex(&mut t),
        "LD" => parse_ld(&mut t),
        "FR" => parse_fr(&mut t),
        "RP" => parse_rp(&mut t),
        "NE" => parse_ne(&mut t),
        "NH" => parse_nh(&mut t),
        "EN" => Ok(NecCard::En),
        _ => Ok(NecCard::Unknown(mnemonic.to_string())),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Card parsers
// ─────────────────────────────────────────────────────────────────────────────

/// GW — Straight wire. 9 required fields.
fn parse_gw(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Gw(GwCard {
        tag: t.require_uint("ITAG")?,
        segment_count: t.require_uint("NS")?,
        x1: t.require_float("XW1")?,
        y1: t.require_float("YW1")?,
        z1: t.require_float("ZW1")?,
        x2: t.require_float("XW2")?,
        y2: t.require_float("YW2")?,
        z2: t.require_float("ZW2")?,
        radius: t.require_float("RAD")?,
    }))
}

/// GA — Circular arc. 6 required fields.
fn parse_ga(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Ga(GaCard {
        tag: t.require_uint("ITAG")?,
        segment_count: t.require_uint("NS")?,
        arc_radius: t.require_float("RADA")?,
        angle1: t.require_float("ANG1")?,
        angle2: t.require_float("ANG2")?,
        radius: t.require_float("RAD")?,
    }))
}

/// GH — Helix. 7 required fields.
fn parse_gh(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Gh(GhCard {
        tag: t.require_uint("ITAG")?,
        segment_count: t.require_uint("NS")?,
        pitch: t.require_float("S")?,
        total_length: t.require_float("HL")?,
        radius_start: t.require_float("A1")?,
        radius_end: t.require_float("A2")?,
        radius: t.require_float("RAD")?,
    }))
}

/// GM — Geometry move/rotate/replicate.
/// Fields: ITAG NRPT ROX ROY ROZ XS YS ZS [ITS]
/// ITS is optional (defaults to 0); only required when NRPT > 0,
/// which is validated by the router.
fn parse_gm(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Gm(GmCard {
        tag: t.require_uint("ITAG")?,
        n_copies: t.require_uint("NRPT")?,
        rot_x: t.require_float("ROX")?,
        rot_y: t.require_float("ROY")?,
        rot_z: t.require_float("ROZ")?,
        trans_x: t.require_float("XS")?,
        trans_y: t.require_float("YS")?,
        trans_z: t.require_float("ZS")?,
        tag_increment: t.opt_uint(0)?,
    }))
}

/// GS — Global scale. Fields: unused unused XSCALE
fn parse_gs(t: &mut Tokens) -> Result<NecCard, ParseError> {
    t.skip_opt(); // field 1 — not used
    t.skip_opt(); // field 2 — not used
    Ok(NecCard::Gs(GsCard {
        scale: t.require_float("XSCALE")?,
    }))
}

/// GE — Geometry end. GPFLAG is optional (defaults to 0).
fn parse_ge(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Ge(GeCard {
        gpflag: t.opt_int(0)?,
    }))
}

/// GN — Ground definition.
/// Fields: IPERF [NRADL [unused_float [unused_float [EPSE [SIG]]]]]
/// All fields after IPERF are optional trailing fields.
fn parse_gn(t: &mut Tokens) -> Result<NecCard, ParseError> {
    let iperf = t.require_int("IPERF")?;
    let nradl = t.opt_int(0)?;
    t.opt_float(0.0)?; // field 3 — not used
    t.opt_float(0.0)?; // field 4 — not used
    let epse = t.opt_float(0.0)?;
    let sig = t.opt_float(0.0)?;
    Ok(NecCard::Gn(GnCard {
        iperf,
        nradl,
        epse,
        sig,
    }))
}

/// EX — Excitation source.
/// Fields: EXTYPE ITAG ISEG [NEXP [EXREAL [EXIMAG]]]
/// NEXP is not used; EXREAL defaults to 1.0, EXIMAG defaults to 0.0.
fn parse_ex(t: &mut Tokens) -> Result<NecCard, ParseError> {
    let ex_type = t.require_int("EXTYPE")?;
    let tag = t.require_uint("ITAG")?;
    let segment = t.require_uint("ISEG")?;
    t.opt_int(0)?; // NEXP — not used
    let voltage_real = t.opt_float(1.0)?;
    let voltage_imag = t.opt_float(0.0)?;
    Ok(NecCard::Ex(ExCard {
        ex_type,
        tag,
        segment,
        voltage_real,
        voltage_imag,
    }))
}

/// LD — Impedance load. 7 required fields.
fn parse_ld(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Ld(LdCard {
        ld_type: t.require_int("LDTYPE")?,
        tag: t.require_uint("ITAG")?,
        first_seg: t.require_uint("LDTAGF")?,
        last_seg: t.require_uint("LDTAGT")?,
        zlr: t.require_float("ZLR")?,
        zli: t.require_float("ZLI")?,
        zlc: t.require_float("ZLC")?,
    }))
}

/// FR — Frequency sweep.
/// Fields: IFRQ NFRQ unused unused FMHZ [DELFRQ]
/// DELFRQ is optional (defaults to 0.0 for a single frequency).
fn parse_fr(t: &mut Tokens) -> Result<NecCard, ParseError> {
    let ifrq = t.require_int("IFRQ")?;
    let nfrq = t.require_uint("NFRQ")?;
    t.opt_int(0)?; // field 3 — not used
    t.opt_int(0)?; // field 4 — not used
    let fmhz = t.require_float("FMHZ")?;
    let delfrq = t.opt_float(0.0)?;
    Ok(NecCard::Fr(FrCard {
        ifrq,
        nfrq,
        fmhz,
        delfrq,
    }))
}

/// RP — Radiation pattern request. 9 required fields.
fn parse_rp(t: &mut Tokens) -> Result<NecCard, ParseError> {
    Ok(NecCard::Rp(RpCard {
        calc: t.require_int("CALC")?,
        n_theta: t.require_uint("NTHETA")?,
        n_phi: t.require_uint("NPHI")?,
        xnda: t.require_int("XNDA")?,
        theta_start: t.require_float("THETS")?,
        phi_start: t.require_float("PHIS")?,
        d_theta: t.require_float("DTHS")?,
        d_phi: t.require_float("DPHS")?,
        rfld: t.require_float("RFLD")?,
    }))
}

/// NE — Near electric field request.
/// Fields: unused NX NY NZ XO YO ZO DX DY DZ
fn parse_ne(t: &mut Tokens) -> Result<NecCard, ParseError> {
    t.skip_opt(); // field 1 — not used
    Ok(NecCard::Ne(parse_near_field(t)?))
}

/// NH — Near magnetic field request. Same field layout as NE.
fn parse_nh(t: &mut Tokens) -> Result<NecCard, ParseError> {
    t.skip_opt(); // field 1 — not used
    Ok(NecCard::Nh(parse_near_field(t)?))
}

/// Shared field parsing for NE and NH (fields 2–10).
fn parse_near_field(t: &mut Tokens) -> Result<NearFieldCard, ParseError> {
    Ok(NearFieldCard {
        nx: t.require_uint("NX")?,
        ny: t.require_uint("NY")?,
        nz: t.require_uint("NZ")?,
        xo: t.require_float("XO")?,
        yo: t.require_float("YO")?,
        zo: t.require_float("ZO")?,
        dx: t.require_float("DX")?,
        dy: t.require_float("DY")?,
        dz: t.require_float("DZ")?,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tokens — field-by-field reader with error context
// ─────────────────────────────────────────────────────────────────────────────

struct Tokens {
    line_number: usize,
    mnemonic: String,
    /// Field tokens extracted from the portion of the line after the mnemonic.
    tokens: Vec<String>,
    /// Index of the next token to be consumed (0-based).
    pos: usize,
}

impl Tokens {
    fn new(line: &str, line_number: usize, mnemonic: &str) -> Self {
        // Skip the 2-character mnemonic and tokenize the rest.
        let rest = if line.len() > 2 { &line[2..] } else { "" };
        let tokens: Vec<String> = rest
            .split_ascii_whitespace()
            .map(|s| s.to_string())
            .collect();
        Tokens {
            line_number,
            mnemonic: mnemonic.to_string(),
            tokens,
            pos: 0,
        }
    }

    /// 1-based field number of the next token, for error messages.
    fn next_field_number(&self) -> usize {
        self.pos + 1
    }

    fn peek(&self) -> Option<&str> {
        self.tokens.get(self.pos).map(|s| s.as_str())
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    // ── Required field readers ────────────────────────────────────────────

    /// Parse a token as i32, accepting scientific-notation floats that are
    /// whole numbers (e.g. "0.00000E+00" → 0). Some NEC generators write every
    /// field in scientific notation, including integer fields.
    fn parse_int_token(
        &self,
        token: &str,
        field_num: usize,
        field_name: &str,
    ) -> Result<i32, ParseError> {
        if let Ok(v) = token.parse::<i32>() {
            return Ok(v);
        }
        if let Ok(f) = token.parse::<f64>() {
            if f.fract() == 0.0 && f >= i32::MIN as f64 && f <= i32::MAX as f64 {
                return Ok(f as i32);
            }
        }
        Err(ParseError::new(
            ParseErrorKind::FieldParseFailure,
            self.line_number,
            format!(
                "{} card: field {} ({}) cannot be parsed as integer: {:?}",
                self.mnemonic, field_num, field_name, token
            ),
        ))
    }

    /// Consume the next token and parse it as i32. Hard error if absent or
    /// unparseable.
    fn require_int(&mut self, field_name: &str) -> Result<i32, ParseError> {
        let field_num = self.next_field_number();
        match self.peek() {
            None => Err(ParseError::new(
                ParseErrorKind::FieldParseFailure,
                self.line_number,
                format!(
                    "{} card: field {} ({}) is missing",
                    self.mnemonic, field_num, field_name
                ),
            )),
            Some(token) => {
                let result = self.parse_int_token(token, field_num, field_name);
                self.advance();
                result
            }
        }
    }

    /// Consume the next token as a non-negative integer (u32). Hard error if
    /// absent, unparseable, or negative.
    fn require_uint(&mut self, field_name: &str) -> Result<u32, ParseError> {
        let field_num = self.next_field_number();
        let v = self.require_int(field_name)?;
        if v < 0 {
            return Err(ParseError::new(
                ParseErrorKind::FieldParseFailure,
                self.line_number,
                format!(
                    "{} card: field {} ({}) must be non-negative, got {}",
                    self.mnemonic, field_num, field_name, v
                ),
            ));
        }
        Ok(v as u32)
    }

    /// Consume the next token and parse it as f64. Hard error if absent or
    /// unparseable. Handles scientific notation automatically via f64::from_str.
    fn require_float(&mut self, field_name: &str) -> Result<f64, ParseError> {
        let field_num = self.next_field_number();
        match self.peek() {
            None => Err(ParseError::new(
                ParseErrorKind::FieldParseFailure,
                self.line_number,
                format!(
                    "{} card: field {} ({}) is missing",
                    self.mnemonic, field_num, field_name
                ),
            )),
            Some(token) => {
                let result = token.parse::<f64>().map_err(|_| {
                    ParseError::new(
                        ParseErrorKind::FieldParseFailure,
                        self.line_number,
                        format!(
                            "{} card: field {} ({}) cannot be parsed as float: {:?}",
                            self.mnemonic, field_num, field_name, token
                        ),
                    )
                });
                self.advance();
                result
            }
        }
    }

    // ── Optional field readers ────────────────────────────────────────────

    /// If a token is present, parse it as i32 (hard error if unparseable).
    /// If absent, return the default.
    fn opt_int(&mut self, default: i32) -> Result<i32, ParseError> {
        match self.peek() {
            None => Ok(default),
            Some(token) => {
                let field_num = self.next_field_number();
                let result = self.parse_int_token(token, field_num, "(optional)");
                self.advance();
                result
            }
        }
    }

    /// If a token is present, parse it as a non-negative integer (hard error if
    /// unparseable or negative). If absent, return the default.
    fn opt_uint(&mut self, default: u32) -> Result<u32, ParseError> {
        match self.peek() {
            None => Ok(default),
            Some(_) => {
                let v = self.opt_int(default as i32)?;
                if v < 0 {
                    let field_num = self.next_field_number();
                    return Err(ParseError::new(
                        ParseErrorKind::FieldParseFailure,
                        self.line_number,
                        format!(
                            "{} card: field {} must be non-negative, got {}",
                            self.mnemonic, field_num, v
                        ),
                    ));
                }
                Ok(v as u32)
            }
        }
    }

    /// If a token is present, parse it as f64 (hard error if unparseable).
    /// If absent, return the default.
    fn opt_float(&mut self, default: f64) -> Result<f64, ParseError> {
        match self.peek() {
            None => Ok(default),
            Some(token) => {
                let field_num = self.next_field_number();
                let result = token.parse::<f64>().map_err(|_| {
                    ParseError::new(
                        ParseErrorKind::FieldParseFailure,
                        self.line_number,
                        format!(
                            "{} card: field {} cannot be parsed as float: {:?}",
                            self.mnemonic, field_num, token
                        ),
                    )
                });
                self.advance();
                result
            }
        }
    }

    /// Skip the next token if one is present. Does nothing if the line has
    /// no more tokens (for discarding unused fields).
    fn skip_opt(&mut self) {
        if self.peek().is_some() {
            self.advance();
        }
    }
}
