// tests/warnings.rs — V-WARN non-fatal warning cases

use crate::parse;

fn warning_kinds(warnings: &crate::errors::ParseWarnings) -> Vec<&str> {
    warnings.iter().map(|w| w.kind.as_str()).collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-001 — unknown card type
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_001_unknown_card() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "XX 0 0 0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    assert_eq!(sim.mesh_input.wires.len(), 1);
    assert!(
        warning_kinds(&warnings).contains(&"UnknownCard"),
        "expected UnknownCard warning; got: {:?}",
        warning_kinds(&warnings)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-002 — unsupported EX type (plane wave, EXTYPE=1)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_002_unsupported_ex_type() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "EX 1 0 0 0 0.0 90.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    assert_eq!(
        sim.sources.len(),
        0,
        "unsupported EX type should be skipped"
    );
    assert!(
        warning_kinds(&warnings).contains(&"UnsupportedExType"),
        "expected UnsupportedExType warning; got: {:?}",
        warning_kinds(&warnings)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-003 — NRADL > 0 in GN card (radial ground screen, ignored)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_003_nradl_ignored() {
    let deck = concat!(
        "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\n",
        "GN 1 32\n",
        "GE 1\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    assert_eq!(sim.mesh_input.ground.ground_type.as_str(), "PEC");
    assert!(
        warning_kinds(&warnings).contains(&"NradlIgnored"),
        "expected NradlIgnored warning; got: {:?}",
        warning_kinds(&warnings)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-004 — missing EN card
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_004_missing_en() {
    // No EN card, no trailing newline.
    let deck = concat!(
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "FR 0 1 0 0 150.0 0.0\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed despite missing EN");
    assert_eq!(sim.mesh_input.wires.len(), 1);
    assert!(
        warning_kinds(&warnings).contains(&"MissingEnCard"),
        "expected MissingEnCard warning; got: {:?}",
        warning_kinds(&warnings)
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// V-WARN-005 — unsupported card (TL — transmission line)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_warn_005_unsupported_card() {
    let deck = concat!(
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n",
        "GE 0\n",
        "TL 1 6 2 6 50.0 0.0 0.0 0.0 0.0 0.0\n",
        "EN\n",
    );
    let (sim, warnings) = parse(deck).expect("parse should succeed");
    assert_eq!(sim.mesh_input.wires.len(), 1);
    assert!(
        warning_kinds(&warnings).contains(&"UnsupportedCard"),
        "expected UnsupportedCard warning; got: {:?}",
        warning_kinds(&warnings)
    );
}
