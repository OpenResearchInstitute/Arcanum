// tests/fmt.rs — V-FMT input format tolerance tests

use super::approx_eq;
use crate::cards::WireDescription;
use crate::parse;

// Expected values shared by all V-FMT tests.
const EXPECTED_TAG: u32 = 1;
const EXPECTED_NS: u32 = 4;
const EXPECTED_Z1: f64 = -0.25;
const EXPECTED_Z2: f64 = 0.25;
const EXPECTED_RAD: f64 = 0.001;

fn check_wire(wires: &[WireDescription]) {
    assert_eq!(wires.len(), 1);
    let WireDescription::Straight(w) = &wires[0] else {
        panic!("expected StraightWire");
    };
    assert_eq!(w.tag, EXPECTED_TAG);
    assert_eq!(w.segment_count, EXPECTED_NS);
    approx_eq!(w.z1, EXPECTED_Z1);
    approx_eq!(w.z2, EXPECTED_Z2);
    approx_eq!(w.radius, EXPECTED_RAD);
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-001 — free-field format (baseline)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_001_free_field() {
    let deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-002 — column-based format (extra whitespace)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_002_column_based() {
    let deck = concat!(
        "GW     1     4     0.0     0.0    -0.25     0.0     0.0     0.25     0.001\n",
        "GE 0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-003 — tab-delimited fields
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_003_tabs() {
    let deck = "GW\t1\t4\t0.0\t0.0\t-0.25\t0.0\t0.0\t0.25\t0.001\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-004 — scientific notation in float fields
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_004_scientific_notation() {
    let deck = "GW 1 4 0.0 0.0 -2.5E-1 0.0 0.0 2.5E-1 1.0E-3\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-005 — Windows line endings (CRLF)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_005_crlf() {
    let deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\r\nGE 0\r\nEN\r\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-FMT-006 — Scientific notation in integer fields
// Some NEC generators (e.g. 4nec2) write every field in scientific notation,
// including semantically-integer fields such as GM's ITS (tag increment).
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_fmt_006_sci_notation_integer_field() {
    // ITS field written as "0.00000E+00" instead of "0".
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GM 0 0 0.0 0.0 45.0 0.0 0.0 0.0 0.00000E+00\n",
        "GE 0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    check_wire(&sim.mesh_input.wires);
    assert!(warnings.is_empty());
}

#[test]
fn v_fmt_006_non_whole_float_in_integer_field_is_error() {
    // A float with a fractional part in an integer field must still be rejected.
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GM 0 0 0.0 0.0 45.0 0.0 0.0 0.0 1.5E+00\n",
        "GE 0\n",
        "EN\n",
    );
    assert!(parse(deck).is_err());
}
