"""
NEC import — hard error cases (V-ERR).
See docs/nec-import/validation.md section 6.

Each test confirms that arcanum.parse() raises ParseError with the expected
error kind. All cases correspond 1:1 to the V-ERR cases in validation.md.
"""

import pytest
import arcanum


# V-ERR-001 — missing GE card
def test_err_missing_ge():
    deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nEN\n"
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "MissingGeCard"


# V-ERR-002 — NS = 0 on GW card
def test_err_ns_zero():
    deck = "GW 1 0 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\nGE 0\nEN\n"
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "ZeroSegmentCount"


# V-ERR-003 — zero-length wire
def test_err_zero_length_wire():
    deck = "GW 1 4 0.5 0.5 0.5 0.5 0.5 0.5 0.001\nGE 0\nEN\n"
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "ZeroLengthWire"


# V-ERR-004 — duplicate ITAG
def test_err_duplicate_tag():
    deck = (
        "GW 1 4 0.0 0.0 -0.5 0.0 0.0 0.0 0.001\n"
        "GW 1 4 0.0 0.0  0.0 0.0 0.0 0.5 0.001\n"
        "GE 0\n"
        "EN\n"
    )
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "DuplicateTag"


# V-ERR-005 — EX references unknown tag
def test_err_ex_unknown_tag():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 99 2 0 1.0 0.0\n"
        "EN\n"
    )
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "UnknownTagReference"


# V-ERR-006 — EX segment number out of range
def test_err_ex_segment_out_of_range():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "EX 0 1 9 0 1.0 0.0\n"
        "EN\n"
    )
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "SegmentOutOfRange"


# V-ERR-007 — non-numeric field
def test_err_non_numeric_field():
    deck = "GW 1 4 0.0 0.0 -0.25 0.0 0.0 OOPS 0.001\nGE 0\nEN\n"
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "FieldParseFailure"


# V-ERR-008 — multiple GN cards
def test_err_multiple_gn():
    deck = (
        "GW 1 4 0.0 0.0 0.0 0.0 0.0 0.5 0.001\n"
        "GN 1\n"
        "GN 2 0 0 0 13.0 0.005\n"
        "GE 1\n"
        "EN\n"
    )
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "MultipleGnCards"


# V-ERR-009 — geometry card after GE
def test_err_geometry_after_ge():
    deck = (
        "GW 1 4 0.0 0.0 -0.25 0.0 0.0 0.25 0.001\n"
        "GE 0\n"
        "GW 2 4 1.0 0.0 -0.25 1.0 0.0 0.25 0.001\n"
        "EN\n"
    )
    with pytest.raises(arcanum.ParseError) as exc_info:
        arcanum.parse(deck)
    assert exc_info.value.kind == "GeometryAfterGe"
