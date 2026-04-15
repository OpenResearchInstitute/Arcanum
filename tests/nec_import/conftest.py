"""
Fixtures for NEC import integration tests.
"""

import pathlib
import pytest

REFERENCE_DECKS = (
    pathlib.Path(__file__).parent.parent.parent
    / "docs" / "nec-import" / "reference-decks"
)


@pytest.fixture
def reference_deck():
    """Return the text of a reference .nec deck by filename stem.

    Usage:
        def test_something(reference_deck):
            text = reference_deck("half-wave-dipole")
    """
    def _load(stem: str) -> str:
        path = REFERENCE_DECKS / f"{stem}.nec"
        if not path.exists():
            raise FileNotFoundError(f"Reference deck not found: {path}")
        return path.read_text(encoding="utf-8")
    return _load
