"""
NEC import — real-world reference deck tests (V-REAL).
See docs/nec-import/validation.md section 8.

Each test loads a committed .nec file from docs/nec-import/reference-decks/
and asserts that the parser produces the correct structure. These tests
exercise the full parser on realistic input rather than constructed minimal cases.
"""

import pytest
import arcanum


# V-REAL-001 — classic half-wave dipole
def test_real_half_wave_dipole(reference_deck):
    sim, warnings = arcanum.parse(reference_deck("half-wave-dipole"))
    assert len(sim.mesh_input.wires) == 1
    wire = sim.mesh_input.wires[0]
    assert wire.tag == 1
    assert wire.segment_count == 11
    assert len(sim.sources) == 1
    assert sim.sources[0].tag == 1
    assert sim.sources[0].segment == 6
    assert len(sim.frequencies) == 1
    assert sim.frequencies[0] == pytest.approx(299_792_458.0)
    assert not warnings


# V-REAL-002 — 3-element Yagi-Uda
def test_real_yagi_3el(reference_deck):
    sim, warnings = arcanum.parse(reference_deck("yagi-3el"))
    assert len(sim.mesh_input.wires) == 3
    tags = {w.tag for w in sim.mesh_input.wires}
    assert tags == {1, 2, 3}
    seg_counts = {w.tag: w.segment_count for w in sim.mesh_input.wires}
    assert all(n == 9 for n in seg_counts.values())
    assert len(sim.sources) == 1
    assert sim.sources[0].tag == 2
    assert sim.sources[0].segment == 5
    assert not warnings


# V-REAL-003 — axial-mode helix over ground plane
def test_real_helix_axial(reference_deck):
    sim, warnings = arcanum.parse(reference_deck("helix-axial"))
    assert len(sim.mesh_input.wires) == 1
    wire = sim.mesh_input.wires[0]
    assert wire.tag == 1
    assert wire.segment_count == 40
    assert wire.n_turns == pytest.approx(5.0)
    assert sim.mesh_input.ground.ground_type == "PEC"
    assert len(sim.sources) == 1
    assert sim.sources[0].tag == 1
    assert sim.sources[0].segment == 1
    assert not warnings


# V-REAL-004 — ORI dumbbell antenna (dense junction network)
def test_real_dumbbell_ori(reference_deck):
    sim, warnings = arcanum.parse(reference_deck("dumbbell-ori"))
    assert len(sim.mesh_input.wires) == 42
    tags = {w.tag for w in sim.mesh_input.wires}
    assert len(tags) == 42
    assert min(tags) == 1
    assert max(tags) == 42
    assert len(sim.sources) == 1
    assert sim.sources[0].tag == 1
    assert sim.sources[0].segment == 1
    assert len(sim.frequencies) == 1
    assert sim.frequencies[0] == pytest.approx(14.2e6)
    assert not warnings
