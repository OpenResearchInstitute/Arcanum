"""
Top-level pytest configuration and fixtures for Arcanum integration tests.

Tests are organized by phase under subdirectories:
    tests/nec_import/    — NEC deck parser (Phase 0)
    tests/geometry/      — Geometry discretization (Phase 1)
    tests/matrix_fill/   — Impedance matrix fill (Phase 2)
    tests/matrix_solve/  — LU solve and excitation (Phase 3)
    tests/postprocess/   — Post-processing (Phase 4)
"""

import pathlib
import pytest

# Root of the Arcanum repository, two levels up from this file.
REPO_ROOT = pathlib.Path(__file__).parent.parent
