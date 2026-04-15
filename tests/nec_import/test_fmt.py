"""
NEC import — input format handling tests (V-FMT).
See docs/nec-import/validation.md section 4.

Verifies that the parser produces identical results regardless of whitespace
style, column alignment, tab characters, scientific notation, or line endings.
All cases correspond 1:1 to the V-FMT cases in validation.md.
"""

import pytest
import arcanum

# Reference result: GW tag=1, NS=4, endpoints and radius matching V-FMT-001.
EXPECTED_TAG = 1
EXPECTED_NS = 4
EXPECTED_Z1 = -0.25
EXPECTED_Z2 = 0.25
EXPECTED_RAD = 0.001


def _check_wire(wire):
    assert wire.tag == EXPECTED_TAG
    assert wire.segment_count == EXPECTED_NS
    assert wire.z1 == pytest.approx(EXPECTED_Z1)
    assert wire.z2 == pytest.approx(EXPECTED_Z2)
    assert wire.radius == pytest.approx(EXPECTED_RAD)


# V-FMT-001 — free-field format (baseline)
def test_fmt_free_field():
    deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    _check_wire(sim.mesh_input.wires[0])
    assert not warnings


# V-FMT-002 — column-based format
def test_fmt_column_based():
    deck = (
        "GW     1     4     0.0     0.0    -0.25     0.0     0.0     0.25     0.001\n"
        "GE 0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    _check_wire(sim.mesh_input.wires[0])
    assert not warnings


# V-FMT-003 — tab-delimited
def test_fmt_tabs():
    deck = "GW\t1\t4\t0.0\t0.0\t-0.25\t0.0\t0.0\t0.25\t0.001\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    _check_wire(sim.mesh_input.wires[0])
    assert not warnings


# V-FMT-004 — scientific notation in float fields
def test_fmt_scientific_notation():
    deck = "GW 1 4 0.0 0.0 -2.5E-1 0.0 0.0 2.5E-1 1.0E-3\nGE 0\nEN\n"
    sim, warnings = arcanum.parse(deck)
    _check_wire(sim.mesh_input.wires[0])
    assert not warnings


# V-FMT-005 — Windows line endings (CRLF)
def test_fmt_crlf():
    deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\r\nGE 0\r\nEN\r\n"
    sim, warnings = arcanum.parse(deck)
    _check_wire(sim.mesh_input.wires[0])
    assert not warnings
