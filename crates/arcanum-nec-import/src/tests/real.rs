// tests/real.rs — V-REAL real-world reference deck tests

use super::approx_eq;
use crate::parse;

// Reference deck content embedded at compile time.
// Paths are relative to this source file's location.
const HALF_WAVE_DIPOLE: &str =
    include_str!("../../../../docs/nec-import/reference-decks/half-wave-dipole.nec");
const YAGI_3EL: &str = include_str!("../../../../docs/nec-import/reference-decks/yagi-3el.nec");
const HELIX_AXIAL: &str =
    include_str!("../../../../docs/nec-import/reference-decks/helix-axial.nec");
const DUMBBELL_ORI: &str =
    include_str!("../../../../docs/nec-import/reference-decks/dumbbell-ori.nec");

// ─────────────────────────────────────────────────────────────────────────────
// V-REAL-001 — classic half-wave dipole
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_real_001_half_wave_dipole() {
    let (sim, warnings) = parse(HALF_WAVE_DIPOLE).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 1);
    let w = &sim.mesh_input.wires[0];
    assert_eq!(w.tag(), 1);
    assert_eq!(w.segment_count(), 11);

    assert_eq!(sim.sources.len(), 1);
    assert_eq!(sim.sources[0].tag, 1);
    assert_eq!(sim.sources[0].segment, 6);

    assert_eq!(sim.frequencies.len(), 1);
    approx_eq!(sim.frequencies[0], 299_792_458.0);

    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-REAL-002 — 3-element Yagi-Uda
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_real_002_yagi_3el() {
    let (sim, warnings) = parse(YAGI_3EL).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 3);
    let tags: std::collections::HashSet<u32> =
        sim.mesh_input.wires.iter().map(|w| w.tag()).collect();
    assert_eq!(tags, [1, 2, 3].into_iter().collect());

    for w in &sim.mesh_input.wires {
        assert_eq!(
            w.segment_count(),
            9,
            "wire tag={} expected 9 segments",
            w.tag()
        );
    }

    assert_eq!(sim.sources.len(), 1);
    assert_eq!(sim.sources[0].tag, 2);
    assert_eq!(sim.sources[0].segment, 5);

    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-REAL-003 — axial-mode helix over ground plane
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_real_003_helix_axial() {
    use crate::cards::WireDescription;

    let (sim, warnings) = parse(HELIX_AXIAL).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 1);
    let WireDescription::Helix(w) = &sim.mesh_input.wires[0] else {
        panic!("expected HelixWire");
    };
    assert_eq!(w.tag, 1);
    assert_eq!(w.segment_count, 40);
    approx_eq!(w.n_turns, 5.0, 1e-6);

    assert_eq!(sim.mesh_input.ground.ground_type.as_str(), "PEC");

    assert_eq!(sim.sources.len(), 1);
    assert_eq!(sim.sources[0].tag, 1);
    assert_eq!(sim.sources[0].segment, 1);

    assert!(warnings.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────────
// V-REAL-004 — ORI dumbbell antenna (dense junction network)
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn v_real_004_dumbbell_ori() {
    let (sim, warnings) = parse(DUMBBELL_ORI).expect("parse should succeed");

    assert_eq!(sim.mesh_input.wires.len(), 42);
    let tags: std::collections::HashSet<u32> =
        sim.mesh_input.wires.iter().map(|w| w.tag()).collect();
    assert_eq!(tags.len(), 42);
    assert_eq!(*tags.iter().min().unwrap(), 1);
    assert_eq!(*tags.iter().max().unwrap(), 42);

    assert_eq!(sim.sources.len(), 1);
    assert_eq!(sim.sources[0].tag, 1);
    assert_eq!(sim.sources[0].segment, 1);

    assert_eq!(sim.frequencies.len(), 1);
    approx_eq!(sim.frequencies[0], 14.2e6, 1.0);

    assert!(warnings.is_empty());
}
