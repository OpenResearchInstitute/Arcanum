"""
NEC import — field parsing tests (V-PARSE).
See docs/nec-import/validation.md section 3.

Tests call arcanum.parse() and assert on the returned SimulationInput fields.
All cases correspond 1:1 to the V-PARSE cases in validation.md.
"""

import pytest
import arcanum


# V-PARSE-001 — GW card field parsing
def test_parse_gw():
    deck = "GW 3 10 0.0 0.0 -0.25 0.0 0.0 0.25 0.002\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    wire = sim.mesh_input.wires[0]
    assert wire.tag == 3
    assert wire.segment_count == 10
    assert wire.x1 == pytest.approx(0.0)
    assert wire.y1 == pytest.approx(0.0)
    assert wire.z1 == pytest.approx(-0.25)
    assert wire.x2 == pytest.approx(0.0)
    assert wire.y2 == pytest.approx(0.0)
    assert wire.z2 == pytest.approx(0.25)
    assert wire.radius == pytest.approx(0.002)
    assert not warnings


# V-PARSE-002 — GA card field parsing
def test_parse_ga():
    deck = "GA 2 8 0.15 0.0 360.0 0.001\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    wire = sim.mesh_input.wires[0]
    assert wire.tag == 2
    assert wire.segment_count == 8
    assert wire.arc_radius == pytest.approx(0.15)
    assert wire.angle1 == pytest.approx(0.0)
    assert wire.angle2 == pytest.approx(360.0)
    assert wire.radius == pytest.approx(0.001)
    assert not warnings


# V-PARSE-003 — GH card field parsing
def test_parse_gh():
    deck = "GH 1 16 0.0238 0.119 0.0239 0.0239 0.001\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    wire = sim.mesh_input.wires[0]
    assert wire.tag == 1
    assert wire.segment_count == 16
    assert wire.pitch == pytest.approx(0.0238)
    assert wire.total_length == pytest.approx(0.119)
    assert wire.radius_start == pytest.approx(0.0239)
    assert wire.radius_end == pytest.approx(0.0239)
    assert wire.radius == pytest.approx(0.001)
    assert wire.n_turns == pytest.approx(5.0)
    assert not warnings


# V-PARSE-004 — GN card, PEC ground
def test_parse_gn_pec():
    deck = "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\nGN 1\nGE 1\nEN\n"
    sim, warnings = arcanum.parse(deck)
    assert sim.mesh_input.ground.ground_type == "PEC"
    assert not warnings


# V-PARSE-005 — GN card, lossy ground
def test_parse_gn_lossy():
    # IPERF=0 → Lossy (reflection-coefficient approximation).
    deck = "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\nGN 0 0 0 0 13.0 0.005\nGE 1\nEN\n"
    sim, warnings = arcanum.parse(deck)
    assert sim.mesh_input.ground.ground_type == "Lossy"
    assert sim.ground_electrical.permittivity == pytest.approx(13.0)
    assert sim.ground_electrical.conductivity == pytest.approx(0.005)
    assert not warnings


# V-PARSE-006 — EX card, voltage source
def test_parse_ex():
    deck = (
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 1 6 0 1.0 0.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.sources) == 1
    src = sim.sources[0]
    assert src.tag == 1
    assert src.segment == 6
    assert src.voltage_real == pytest.approx(1.0)
    assert src.voltage_imag == pytest.approx(0.0)
    assert not warnings


# V-PARSE-007 — FR card, single frequency
def test_parse_fr_single():
    deck = (
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 1 6 0 1.0 0.0\n"
        "FR 0 1 0 0 299.792458 0.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.frequencies) == 1
    assert sim.frequencies[0] == pytest.approx(299_792_458.0)
    assert not warnings


# V-PARSE-008 — FR card, linear frequency sweep
def test_parse_fr_linear_sweep():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 1 2 0 1.0 0.0\n"
        "FR 0 5 0 0 100.0 50.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.frequencies) == 5
    expected = [100e6, 150e6, 200e6, 250e6, 300e6]
    for got, exp in zip(sim.frequencies, expected):
        assert got == pytest.approx(exp)
    assert not warnings


# V-PARSE-009 — FR card, multiplicative frequency sweep
def test_parse_fr_multiplicative_sweep():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 1 2 0 1.0 0.0\n"
        "FR 1 4 0 0 100.0 2.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.frequencies) == 4
    expected = [100e6, 200e6, 400e6, 800e6]
    for got, exp in zip(sim.frequencies, expected):
        assert got == pytest.approx(exp)
    assert not warnings


# V-PARSE-010 — CM/CE comment cards are silently discarded
def test_parse_comments_discarded():
    deck_with_comments = (
        "CM Arcanum test deck\n"
        "CM Half-wave dipole at 300 MHz\n"
        "CE\n"
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EN\n"
    )
    deck_without_comments = (
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EN\n"
    )
    sim_with, _ = arcanum.parse(deck_with_comments)
    sim_without, _ = arcanum.parse(deck_without_comments)
    assert len(sim_with.mesh_input.wires) == len(sim_without.mesh_input.wires)
    assert sim_with.mesh_input.wires[0].tag == sim_without.mesh_input.wires[0].tag
