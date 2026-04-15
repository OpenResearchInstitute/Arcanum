// errors.rs
//
// ParseError and ParseWarnings — the two error-reporting types for the
// NEC import pipeline.
//
// ParseError is returned on hard failure; parsing aborts immediately.
// ParseWarnings is accumulated throughout parsing and returned alongside
// a successful SimulationInput. Warnings never abort parsing.
//
// Matches docs/nec-import/design.md Section 5 exactly.

use std::fmt;

// ─────────────────────────────────────────────────────────────────────────────
// ParseError
// ─────────────────────────────────────────────────────────────────────────────

/// A fatal parse error that aborts parsing immediately.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    /// 1-based line number in the input where the error was detected.
    pub line: usize,
    /// Human-readable description including card type and field context.
    pub message: String,
}

impl ParseError {
    pub fn new(kind: ParseErrorKind, line: usize, message: impl Into<String>) -> Self {
        ParseError {
            kind,
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] {:?}: {}", self.line, self.kind, self.message)
    }
}

impl std::error::Error for ParseError {}

/// Discriminant for ParseError, used by callers to match on error category.
/// String representation (via as_str) is used by the Python test suite.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseErrorKind {
    /// GE card is absent from the deck.
    MissingGeCard,
    /// Two geometry cards share the same tag number.
    DuplicateTag,
    /// NS = 0 on a geometry card.
    ZeroSegmentCount,
    /// Wire endpoints are identical (zero-length wire).
    ZeroLengthWire,
    /// EX or LD references a tag not present in the mesh.
    UnknownTagReference,
    /// EX or LD segment number exceeds NS for the referenced wire.
    SegmentOutOfRange,
    /// A field could not be parsed as the expected type (int or float).
    FieldParseFailure,
    /// A field parsed successfully but its value is out of the allowed range.
    InvalidFieldValue,
    /// More than one GN card is present in the deck.
    MultipleGnCards,
    /// A geometry card (GW, GA, GH, GM, GS) appears after GE.
    GeometryAfterGe,
}

impl ParseErrorKind {
    /// String representation used by the Python interface for kind comparisons.
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseErrorKind::MissingGeCard => "MissingGeCard",
            ParseErrorKind::DuplicateTag => "DuplicateTag",
            ParseErrorKind::ZeroSegmentCount => "ZeroSegmentCount",
            ParseErrorKind::ZeroLengthWire => "ZeroLengthWire",
            ParseErrorKind::UnknownTagReference => "UnknownTagReference",
            ParseErrorKind::SegmentOutOfRange => "SegmentOutOfRange",
            ParseErrorKind::FieldParseFailure => "FieldParseFailure",
            ParseErrorKind::InvalidFieldValue => "InvalidFieldValue",
            ParseErrorKind::MultipleGnCards => "MultipleGnCards",
            ParseErrorKind::GeometryAfterGe => "GeometryAfterGe",
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ParseWarning and ParseWarnings
// ─────────────────────────────────────────────────────────────────────────────

/// A single non-fatal warning collected during parsing.
#[derive(Debug, Clone)]
pub struct ParseWarning {
    pub kind: ParseWarningKind,
    /// 1-based line number where the condition was detected.
    pub line: usize,
    pub message: String,
}

impl ParseWarning {
    pub fn new(kind: ParseWarningKind, line: usize, message: impl Into<String>) -> Self {
        ParseWarning {
            kind,
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ParseWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] {:?}: {}", self.line, self.kind, self.message)
    }
}

/// Discriminant for ParseWarning.
#[derive(Debug, Clone, PartialEq)]
pub enum ParseWarningKind {
    /// A card mnemonic was not recognized.
    UnknownCard,
    /// EX card has an unsupported EXTYPE (not 0 or 5). Card is skipped.
    UnsupportedExType,
    /// A card type is recognized but not yet implemented. Card is skipped.
    UnsupportedCard,
    /// GN card has NRADL > 0 (radial ground screen). Value is ignored.
    NradlIgnored,
    /// EN card is absent. Deck is otherwise valid.
    MissingEnCard,
    /// Two wire endpoints are very close but not connected (< 0.001 × shortest segment).
    NearCoincidentEndpoints,
    /// A wire segment endpoint is at or below the ground plane (z ≤ 0) when a
    /// ground plane is present.
    WireInGroundPlane,
}

impl ParseWarningKind {
    /// String representation used by the Python interface for kind comparisons.
    pub fn as_str(&self) -> &'static str {
        match self {
            ParseWarningKind::UnknownCard => "UnknownCard",
            ParseWarningKind::UnsupportedExType => "UnsupportedExType",
            ParseWarningKind::UnsupportedCard => "UnsupportedCard",
            ParseWarningKind::NradlIgnored => "NradlIgnored",
            ParseWarningKind::MissingEnCard => "MissingEnCard",
            ParseWarningKind::NearCoincidentEndpoints => "NearCoincidentEndpoints",
            ParseWarningKind::WireInGroundPlane => "WireInGroundPlane",
        }
    }
}

/// The accumulated collection of non-fatal warnings from a parse run.
#[derive(Debug, Clone, Default)]
pub struct ParseWarnings(Vec<ParseWarning>);

impl ParseWarnings {
    pub fn new() -> Self {
        ParseWarnings(Vec::new())
    }

    pub fn push(&mut self, warning: ParseWarning) {
        self.0.push(warning);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ParseWarning> {
        self.0.iter()
    }

    /// Consume and return the inner Vec, used by arcanum-py to build a Python list.
    pub fn into_vec(self) -> Vec<ParseWarning> {
        self.0
    }
}

impl IntoIterator for ParseWarnings {
    type Item = ParseWarning;
    type IntoIter = std::vec::IntoIter<ParseWarning>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
