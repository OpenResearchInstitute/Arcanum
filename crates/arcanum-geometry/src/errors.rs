// errors.rs — Phase 1 error and warning types

/// A hard error that aborts mesh construction.
#[derive(Debug, Clone)]
pub struct GeometryError {
    pub kind: GeometryErrorKind,
    /// 1-based index into the wire list (not a line number — Phase 1 works
    /// with already-parsed WireDescriptions).
    pub wire_index: usize,
    pub message: String,
}

impl GeometryError {
    pub fn new(kind: GeometryErrorKind, wire_index: usize, message: impl Into<String>) -> Self {
        GeometryError {
            kind,
            wire_index,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for GeometryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[wire {}] {}: {}",
            self.wire_index,
            self.kind.as_str(),
            self.message
        )
    }
}

/// Category of a hard geometry error.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryErrorKind {
    /// Wire has zero length (start == end).
    ZeroLengthWire,
    /// Segment count is zero (should have been caught by nec-import, but
    /// checked here defensively).
    ZeroSegmentCount,
    /// A GM operation references a tag that does not exist in the wire list.
    UnknownTagReference,
    /// A GM copy operation would generate a duplicate tag.
    DuplicateTag,
    /// A coordinate is NaN or infinite.
    InvalidCoordinate,
}

impl GeometryErrorKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            GeometryErrorKind::ZeroLengthWire => "ZeroLengthWire",
            GeometryErrorKind::ZeroSegmentCount => "ZeroSegmentCount",
            GeometryErrorKind::UnknownTagReference => "UnknownTagReference",
            GeometryErrorKind::DuplicateTag => "DuplicateTag",
            GeometryErrorKind::InvalidCoordinate => "InvalidCoordinate",
        }
    }
}

/// A non-fatal condition worth reporting.
#[derive(Debug, Clone)]
pub struct GeometryWarning {
    pub kind: GeometryWarningKind,
    pub message: String,
}

impl GeometryWarning {
    pub fn new(kind: GeometryWarningKind, message: impl Into<String>) -> Self {
        GeometryWarning {
            kind,
            message: message.into(),
        }
    }
}

/// Category of a geometry warning.
#[derive(Debug, Clone, PartialEq)]
pub enum GeometryWarningKind {
    /// Two wire endpoints are closer than 10ε but farther than ε — possible
    /// modeling error.
    NearCoincidentEndpoints,
    /// A wire lies entirely in the z = 0 ground plane; no image generated.
    WireInGroundPlane,
}

impl GeometryWarningKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            GeometryWarningKind::NearCoincidentEndpoints => "NearCoincidentEndpoints",
            GeometryWarningKind::WireInGroundPlane => "WireInGroundPlane",
        }
    }
}

/// Accumulated list of geometry warnings.
#[derive(Debug, Clone, Default)]
pub struct GeometryWarnings(Vec<GeometryWarning>);

impl GeometryWarnings {
    pub fn new() -> Self {
        GeometryWarnings::default()
    }

    pub fn push(&mut self, w: GeometryWarning) {
        self.0.push(w);
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_vec(self) -> Vec<GeometryWarning> {
        self.0
    }

    pub fn iter(&self) -> impl Iterator<Item = &GeometryWarning> {
        self.0.iter()
    }
}
