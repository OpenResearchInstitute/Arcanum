// tests/errors.rs — V-ERR hard error cases

use crate::errors::ParseErrorKind;
use crate::parse;

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-001 — missing GE card
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_001_missing_ge() {
    let deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nEN\n";
    let err = parse(deck).expect_err("should fail with MissingGeCard");
    assert_eq!(err.kind, ParseErrorKind::MissingGeCard);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-002 — NS = 0 on GW card
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_002_ns_zero() {
    let deck = "GW 1 0 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nGE 0\nEN\n";
    let err = parse(deck).expect_err("should fail with ZeroSegmentCount");
    assert_eq!(err.kind, ParseErrorKind::ZeroSegmentCount);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-003 — zero-length wire
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_003_zero_length_wire() {
    let deck = "GW 1 4 0.5 0.5 0.5 0.5 0.5 0.5 0.001\nGE 0\nEN\n";
    let err = parse(deck).expect_err("should fail with ZeroLengthWire");
    assert_eq!(err.kind, ParseErrorKind::ZeroLengthWire);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-004 — duplicate ITAG
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_004_duplicate_tag() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.5 0.0 0.0 0.0 0.001\n",
        "GW 1 4 0.0 0.0  0.0 0.0 0.0 0.5 0.001\n",
        "GE 0\n",
        "EN\n",
    );
    let err = parse(deck).expect_err("should fail with DuplicateTag");
    assert_eq!(err.kind, ParseErrorKind::DuplicateTag);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-005 — EX references unknown tag
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_005_ex_unknown_tag() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 99 2 0 1.0 0.0\n",
        "EN\n",
    );
    let err = parse(deck).expect_err("should fail with UnknownTagReference");
    assert_eq!(err.kind, ParseErrorKind::UnknownTagReference);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-006 — EX segment number out of range
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_006_ex_segment_out_of_range() {
    // Wire has NS=4; segment 9 > 4.
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 1 9 0 1.0 0.0\n",
        "EN\n",
    );
    let err = parse(deck).expect_err("should fail with SegmentOutOfRange");
    assert_eq!(err.kind, ParseErrorKind::SegmentOutOfRange);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-007 — non-numeric field
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_007_non_numeric_field() {
    let deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 OOPS 0.001\nGE 0\nEN\n";
    let err = parse(deck).expect_err("should fail with FieldParseFailure");
    assert_eq!(err.kind, ParseErrorKind::FieldParseFailure);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-008 — multiple GN cards
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_008_multiple_gn() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\n",
        "GN 1\n",
        "GN 2 0 0 0 13.0 0.005\n",
        "GE 1\n",
        "EN\n",
    );
    let err = parse(deck).expect_err("should fail with MultipleGnCards");
    assert_eq!(err.kind, ParseErrorKind::MultipleGnCards);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-ERR-009 — geometry card after GE
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_err_009_geometry_after_ge() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "GW 2 4 1.0 0.0 -0.25 1.0 0.0 0.25 0.001\n",
        "EN\n",
    );
    let err = parse(deck).expect_err("should fail with GeometryAfterGe");
    assert_eq!(err.kind, ParseErrorKind::GeometryAfterGe);
}
