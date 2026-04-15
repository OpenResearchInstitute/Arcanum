// tests/parse.rs — V-PARSE field parsing tests

use super::approx_eq;
use crate::cards::WireDescription;
use crate::parse;

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-001 — GW card field parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_001_gw_fields() {
    let deck = "GW 3 10 0.0 0.0 -0.25 0.0 0.0 0.25 0.002\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 1);
    let WireDescription::Straight(w) = &sim.mesh_input.wires[0] else {
        panic!("expected StraightWire");
    };
    assert_eq!(w.tag, 3);
    assert_eq!(w.segment_count, 10);
    approx_eq!(w.x1, 0.0);
    approx_eq!(w.y1, 0.0);
    approx_eq!(w.z1, -0.25);
    approx_eq!(w.x2, 0.0);
    approx_eq!(w.y2, 0.0);
    approx_eq!(w.z2, 0.25);
    approx_eq!(w.radius, 0.002);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-002 — GA card field parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_002_ga_fields() {
    let deck = "GA 2 8 0.15 0.0 360.0 0.001\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 1);
    let WireDescription::Arc(w) = &sim.mesh_input.wires[0] else {
        panic!("expected ArcWire");
    };
    assert_eq!(w.tag, 2);
    assert_eq!(w.segment_count, 8);
    approx_eq!(w.arc_radius, 0.15);
    approx_eq!(w.angle1, 0.0);
    approx_eq!(w.angle2, 360.0);
    approx_eq!(w.radius, 0.001);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-003 — GH card field parsing
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_003_gh_fields() {
    let deck = "GH 1 16 0.0238 0.119 0.0239 0.0239 0.001\nGE 0\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 1);
    let WireDescription::Helix(w) = &sim.mesh_input.wires[0] else {
        panic!("expected HelixWire");
    };
    assert_eq!(w.tag, 1);
    assert_eq!(w.segment_count, 16);
    approx_eq!(w.pitch, 0.0238);
    approx_eq!(w.total_length, 0.119);
    approx_eq!(w.radius_start, 0.0239);
    approx_eq!(w.radius_end, 0.0239);
    approx_eq!(w.radius, 0.001);
    approx_eq!(w.n_turns, 5.0, 1e-6);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-004 — GN card, PEC ground
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_004_gn_pec() {
    let deck = "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\nGN 1\nGE 1\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.mesh_input.ground.ground_type.as_str(), "PEC");
    assert!(sim.ground_electrical.is_none());
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-005 — GN card, lossy ground (IPERF=0)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_005_gn_lossy() {
    // IPERF=0 → Lossy (reflection-coefficient approximation).
    let deck = "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\nGN 0 0 0 0 13.0 0.005\nGE 1\nEN\n";
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.mesh_input.ground.ground_type.as_str(), "Lossy");
    let ge = sim
        .ground_electrical
        .as_ref()
        .expect("ground_electrical should be Some");
    approx_eq!(ge.permittivity, 13.0);
    approx_eq!(ge.conductivity, 0.005);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-006 — EX card, voltage source
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_006_ex_voltage_source() {
    let deck = concat!(
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 1 6 0 1.0 0.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.sources.len(), 1);
    let src = &sim.sources[0];
    assert_eq!(src.tag, 1);
    assert_eq!(src.segment, 6);
    approx_eq!(src.voltage_real, 1.0);
    approx_eq!(src.voltage_imag, 0.0);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-007 — FR card, single frequency
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_007_fr_single() {
    let deck = concat!(
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 1 6 0 1.0 0.0\n",
        "FR 0 1 0 0 299.792458 0.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.frequencies.len(), 1);
    approx_eq!(sim.frequencies[0], 299_792_458.0);
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-008 — FR card, linear frequency sweep
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_008_fr_linear_sweep() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 1 2 0 1.0 0.0\n",
        "FR 0 5 0 0 100.0 50.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.frequencies.len(), 5);
    let expected = [100e6, 150e6, 200e6, 250e6, 300e6];
    for (got, exp) in sim.frequencies.iter().zip(expected.iter()) {
        approx_eq!(*got, *exp, 1.0); // 1 Hz tolerance for MHz→Hz conversion
    }
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-009 — FR card, multiplicative frequency sweep
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_009_fr_multiplicative_sweep() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 0 1 2 0 1.0 0.0\n",
        "FR 1 4 0 0 100.0 2.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");

    assert_eq!(sim.frequencies.len(), 4);
    let expected = [100e6, 200e6, 400e6, 800e6];
    for (got, exp) in sim.frequencies.iter().zip(expected.iter()) {
        approx_eq!(*got, *exp, 1.0);
    }
    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-PARSE-010 — CM/CE comment cards are silently discarded
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_parse_010_comments_discarded() {
    let deck_with_comments = concat!(
        "CM Arcanum test deck\n",
        "CM Half-wave dipole at 300 MHz\n",
        "CE\n",
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EN\n",
    );
    let deck_without_comments = "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nGE 0\nEN\n";

    let (sim_with, _) = parse(deck_with_comments).expect("parse should succeed");
    let (sim_without, _) = parse(deck_without_comments).expect("parse should succeed");

    assert_eq!(
        sim_with.mesh_input.wires.len(),
        sim_without.mesh_input.wires.len()
    );
    assert_eq!(
        sim_with.mesh_input.wires[0].tag(),
        sim_without.mesh_input.wires[0].tag()
    );
}
