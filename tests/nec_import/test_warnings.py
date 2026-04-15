"""
NEC import — warning cases (V-WARN).
See docs/nec-import/validation.md section 7.

Each test confirms that arcanum.parse() succeeds (no exception) and that
ParseWarnings contains an entry with the expected warning kind.
All cases correspond 1:1 to the V-WARN cases in validation.md.
"""

import pytest
import arcanum


def _warning_kinds(warnings):
    return [w.kind for w in warnings]


# V-WARN-001 — unknown card type
def test_warn_unknown_card():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "XX 0 0 0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.mesh_input.wires) == 1
    assert "UnknownCard" in _warning_kinds(warnings)


# V-WARN-002 — unsupported EX type (plane wave)
def test_warn_unsupported_ex_type():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 1 0 0 0 0.0 90.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.sources) == 0
    assert "UnsupportedExType" in _warning_kinds(warnings)


# V-WARN-003 — NRADL > 0 in GN card
def test_warn_nradl_ignored():
    deck = (
        "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\n"
        "GN 1 32\n"
        "GE 1\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert sim.mesh_input.ground.ground_type == "PEC"
    assert "NradlIgnored" in _warning_kinds(warnings)


# V-WARN-004 — missing EN card
def test_warn_missing_en():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "FR 0 1 0 0 150.0 0.0\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.mesh_input.wires) == 1
    assert "MissingEnCard" in _warning_kinds(warnings)


# V-WARN-005 — unsupported card (TL)
def test_warn_unsupported_card():
    deck = (
        "GW 1 11 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "TL 1 6 2 6 50.0 0.0 0.0 0.0 0.0 0.0\n"
        "EN\n"
    )
    sim, warnings = arcanum.parse(deck)
    assert len(sim.mesh_input.wires) == 1
    assert "UnsupportedCard" in _warning_kinds(warnings)
