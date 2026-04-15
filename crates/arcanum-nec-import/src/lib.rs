// arcanum-nec-import
//
// NEC input deck parser. Accepts a .nec file as a string and produces a
// SimulationInput struct consumed independently by each computational phase.
//
// Public API:
//   pub fn parse(input: &str) -> Result<(SimulationInput, ParseWarnings), ParseError>
//   pub fn parse_file(path: &Path) -> Result<(SimulationInput, ParseWarnings), ParseError>

use std::path::Path;

pub mod cards;
pub mod errors;
pub(crate) mod lexer;
pub(crate) mod router;
pub(crate) mod tag_registry;

#[cfg(test)]
mod tests;

/// Parse a NEC input deck from a string.
///
/// Returns `(SimulationInput, ParseWarnings)` on success.
/// Returns `ParseError` immediately on the first hard error.
pub fn parse(input: &str) -> Result<(SimulationInput, ParseWarnings), ParseError> {
    let deck = lexer::lex(input)?;
    router::route(deck)
}

/// Parse a NEC input deck from a file path.
///
/// Reads the file to a string, then calls `parse`. I/O errors are wrapped
/// in a `ParseError` with `ParseErrorKind::FieldParseFailure` and line 0.
pub fn parse_file(path: &Path) -> Result<(SimulationInput, ParseWarnings), ParseError> {
    let input = std::fs::read_to_string(path).map_err(|e| {
        ParseError::new(
            ParseErrorKind::FieldParseFailure,
            0,
            format!("could not read {}: {}", path.display(), e),
        )
    })?;
    parse(&input)
}

// Re-export public types so callers can use arcanum_nec_import::SimulationInput
// without qualifying the module path.
pub use errors::{ParseError, ParseErrorKind, ParseWarning, ParseWarningKind, ParseWarnings};

pub use cards::{
    ArcWire, GeometricGround, GeometryTransforms, GmOperation, GroundElectrical, GroundModel,
    GroundType, HelixWire, LoadDefinition, MeshInput, NearFieldRequest, OutputRequests,
    RadiationPatternRequest, SimulationInput, SourceDefinition, StraightWire, WireDescription,
};
