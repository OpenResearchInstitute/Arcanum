// tag_registry.rs
//
// Tag registry — maps wire tag numbers to wire index and segment count.
//
// Built by the router as geometry cards are processed. Used to validate
// EX and LD card references after all geometry cards have been seen.
//
// Hard errors:
//   - Duplicate tag on insert (two geometry cards with the same ITAG)
//   - Unknown tag on resolve (EX/LD references a tag not in the mesh)
//   - Segment out of range on resolve (ISEG > NS for the referenced wire)

use crate::errors::{ParseError, ParseErrorKind};
use std::collections::HashMap;

// ─────────────────────────────────────────────────────────────────────────────
// TagEntry
// ─────────────────────────────────────────────────────────────────────────────

/// Everything the router needs to know about a registered wire tag.
#[derive(Debug, Clone)]
pub(crate) struct TagEntry {
    /// Index into MeshInput.wires for this tag. Reserved for Phase 1 use.
    #[allow(dead_code)]
    pub wire_index: usize,
    /// Segment count (NS) from the geometry card.
    pub segment_count: u32,
    /// Source line number, preserved for duplicate-tag error messages.
    pub line_number: usize,
}

// ─────────────────────────────────────────────────────────────────────────────
// TagRegistry
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
pub(crate) struct TagRegistry {
    entries: HashMap<u32, TagEntry>,
}

impl TagRegistry {
    pub fn new() -> Self {
        TagRegistry {
            entries: HashMap::new(),
        }
    }

    /// Register a new wire tag.
    ///
    /// Returns a hard error if `tag` has already been registered, identifying
    /// both the current line and the line of the first registration.
    pub fn insert(
        &mut self,
        tag: u32,
        wire_index: usize,
        segment_count: u32,
        line_number: usize,
    ) -> Result<(), ParseError> {
        if let Some(existing) = self.entries.get(&tag) {
            return Err(ParseError::new(
                ParseErrorKind::DuplicateTag,
                line_number,
                format!(
                    "tag {} already defined at line {}; duplicate definition at line {}",
                    tag, existing.line_number, line_number
                ),
            ));
        }
        self.entries.insert(
            tag,
            TagEntry {
                wire_index,
                segment_count,
                line_number,
            },
        );
        Ok(())
    }

    /// Look up a tag and validate a segment number against it.
    ///
    /// Returns a reference to the TagEntry on success.
    ///
    /// Hard errors:
    ///   - Tag not found → UnknownTagReference
    ///   - segment == 0 or segment > NS → SegmentOutOfRange
    pub fn resolve(
        &self,
        tag: u32,
        segment: u32,
        line_number: usize,
    ) -> Result<&TagEntry, ParseError> {
        let entry = self.entries.get(&tag).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnknownTagReference,
                line_number,
                format!("tag {} is not defined in the geometry section", tag),
            )
        })?;

        if segment == 0 || segment > entry.segment_count {
            return Err(ParseError::new(
                ParseErrorKind::SegmentOutOfRange,
                line_number,
                format!(
                    "segment {} is out of range for tag {} (NS = {}); \
                     segments are numbered 1 through {}",
                    segment, tag, entry.segment_count, entry.segment_count
                ),
            ));
        }

        Ok(entry)
    }

    /// Look up a tag without validating a segment number.
    ///
    /// Used by LD cards, which reference a range of segments validated
    /// separately by the router.
    pub fn get(&self, tag: u32, line_number: usize) -> Result<&TagEntry, ParseError> {
        self.entries.get(&tag).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnknownTagReference,
                line_number,
                format!("tag {} is not defined in the geometry section", tag),
            )
        })
    }

    /// Returns true if no tags have been registered yet.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}
