# NEC Import — Implementation Plan

**Project:** Arcanum  
**Document:** `docs/nec-import/plan.md`  
**Status:** DRAFT  
**Revision:** 0.2

---

## Overview

The implementation is a Cargo workspace containing one Rust crate per computational
phase, plus a dedicated PyO3 binding crate. This structure keeps each phase's
dependencies isolated, allows phases to be developed and tested independently, and
mirrors the clean phase boundaries described in `design.md`.

Tests run at two levels: Rust unit tests within each crate (covering logic directly)
and Python pytest scripts organized by phase (exercising the PyO3 boundary and
running the reference decks). A GitHub Actions workflow runs both on every push
and PR.

The Python-facing interface uses **rich objects**: `#[pyclass]`-decorated structs
with attribute access, matching the strongly-typed phase boundaries in the design
and producing self-documenting test assertions.

---

## Repository structure to be created

```
Arcanum/
├── Cargo.toml                              ← workspace root (no code here)
├── pyproject.toml                          ← maturin config, points to arcanum-py
│
├── crates/
│   ├── arcanum-nec-import/                 ← NEC deck parser (rlib)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                      ← pub fn parse(), pub fn parse_file()
│   │       ├── cards.rs                    ← NecCard enum, all card structs
│   │       ├── lexer.rs                    ← Stage 1: text → ParsedDeck
│   │       ├── router.rs                   ← Stage 2: ParsedDeck → SimulationInput
│   │       ├── tag_registry.rs             ← tag → wire index + segment count
│   │       ├── errors.rs                   ← ParseError, ParseWarnings
│   │       └── tests/
│   │           ├── mod.rs
│   │           ├── parse_tests.rs          ← V-PARSE-001 through V-PARSE-010
│   │           ├── fmt_tests.rs            ← V-FMT-001 through V-FMT-005
│   │           ├── route_tests.rs          ← V-ROUTE-001 through V-ROUTE-002
│   │           ├── error_tests.rs          ← V-ERR-001 through V-ERR-009
│   │           └── warn_tests.rs           ← V-WARN-001 through V-WARN-005
│   │
│   ├── arcanum-geometry/                   ← Phase 1: geometry discretization (rlib)
│   │   ├── Cargo.toml                        depends on: arcanum-nec-import, nalgebra
│   │   └── src/
│   │       ├── lib.rs
│   │       └── tests/
│   │           └── mod.rs
│   │
│   ├── arcanum-matrix-fill/                ← Phase 2: impedance matrix (rlib)
│   │   ├── Cargo.toml                        depends on: arcanum-geometry, rayon, gauss-quad
│   │   └── src/
│   │       ├── lib.rs
│   │       └── tests/
│   │           └── mod.rs
│   │
│   ├── arcanum-matrix-solve/               ← Phase 3: LU solve + excitation (rlib)
│   │   ├── Cargo.toml                        depends on: arcanum-matrix-fill, faer
│   │   └── src/
│   │       ├── lib.rs
│   │       └── tests/
│   │           └── mod.rs
│   │
│   ├── arcanum-postprocess/                ← Phase 4: patterns, near fields (rlib)
│   │   ├── Cargo.toml                        depends on: arcanum-matrix-solve, nalgebra
│   │   └── src/
│   │       ├── lib.rs
│   │       └── tests/
│   │           └── mod.rs
│   │
│   └── arcanum-py/                         ← PyO3 bindings only (cdylib)
│       ├── Cargo.toml                        depends on: all five crates above, pyo3
│       └── src/
│           └── lib.rs                      ← #[pymodule], #[pyfunction], #[pyclass] wrappers
│
├── python/
│   └── arcanum/
│       ├── __init__.py                     ← imports compiled extension, re-exports symbols
│       ├── nec_import.py                   ← Python helpers and type stubs for nec-import
│       ├── geometry.py                     ← [Phase 1, future]
│       ├── matrix_fill.py                  ← [Phase 2, future]
│       ├── matrix_solve.py                 ← [Phase 3, future]
│       └── postprocess.py                  ← [Phase 4, future]
│
├── tests/
│   ├── conftest.py                         ← top-level fixtures (project root path, etc.)
│   ├── nec_import/
│   │   ├── conftest.py                     ← reference_deck() fixture
│   │   ├── test_parse.py                   ← V-PARSE cases via PyO3
│   │   ├── test_fmt.py                     ← V-FMT cases via PyO3
│   │   ├── test_errors.py                  ← V-ERR cases via PyO3
│   │   ├── test_warnings.py                ← V-WARN cases via PyO3
│   │   └── test_real.py                    ← V-REAL-001 through V-REAL-004
│   ├── geometry/
│   │   └── .gitkeep                        ← [Phase 1, future]
│   ├── matrix_fill/
│   │   └── .gitkeep                        ← [Phase 2, future]
│   ├── matrix_solve/
│   │   └── .gitkeep                        ← [Phase 3, future]
│   └── postprocess/
│       └── .gitkeep                        ← [Phase 4, future]
│
└── .github/
    └── workflows/
        └── ci.yml
```

---

## Step 1 — Project scaffold

### Workspace `Cargo.toml` (root — no `[package]`, no code)

```toml
[workspace]
members = [
    "crates/arcanum-nec-import",
    "crates/arcanum-geometry",
    "crates/arcanum-matrix-fill",
    "crates/arcanum-matrix-solve",
    "crates/arcanum-postprocess",
    "crates/arcanum-py",
]
resolver = "2"

[workspace.dependencies]
pyo3     = { version = "0.22", features = ["extension-module"] }
nalgebra = "0.33"
rayon    = "1.10"
faer     = "0.19"
```

### `crates/arcanum-nec-import/Cargo.toml`

```toml
[package]
name = "arcanum-nec-import"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[dependencies]
# no external dependencies — intentional; pure parsing logic only
```

### `crates/arcanum-py/Cargo.toml`

```toml
[package]
name = "arcanum-py"
version = "0.1.0"
edition = "2021"

[lib]
name = "arcanum"           # the Python module name
crate-type = ["cdylib"]

[dependencies]
arcanum-nec-import  = { path = "../arcanum-nec-import" }
arcanum-geometry    = { path = "../arcanum-geometry" }
arcanum-matrix-fill = { path = "../arcanum-matrix-fill" }
arcanum-matrix-solve = { path = "../arcanum-matrix-solve" }
arcanum-postprocess = { path = "../arcanum-postprocess" }
pyo3 = { workspace = true }
```

### Root `pyproject.toml`

```toml
[build-system]
requires = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[project]
name = "arcanum"
requires-python = ">=3.9"

[tool.maturin]
manifest-path = "crates/arcanum-py/Cargo.toml"
features = ["pyo3/extension-module"]
python-source = "python"
```

---

## Step 2 — Data types (`crates/arcanum-nec-import/src/cards.rs`)

Define all card structs as plain Rust structs — no PyO3 on these. They are internal
to the `arcanum-nec-import` crate. The PyO3 binding crate wraps them separately.

Structs to define:
- One per card type: `GwCard`, `GaCard`, `GhCard`, `GmCard`, `GsCard`, `GeCard`,
  `GnCard`, `ExCard`, `LdCard`, `FrCard`, `RpCard`, `NeCard`, `NhCard`
- `NecCard` enum with variants for each card type plus `Unknown(String)`
- `ParsedDeck` — `Vec<(usize, NecCard)>` with line numbers preserved
- `SimulationInput`, `MeshInput`, `WireDescription` (union of straight/arc/helix),
  `GeometricGround`, `GroundElectrical`, `SourceDefinition`, `LoadDefinition`,
  `OutputRequests`

These match `design.md` Sections 3.2, 4.2, and 4.3 exactly.

---

## Step 3 — Error types (`crates/arcanum-nec-import/src/errors.rs`)

Implement `ParseError` (kind + line + message) and `ParseWarnings` (Vec of
`ParseWarning`) matching `design.md` Section 5.

`ParseErrorKind` and `ParseWarningKind` are Rust enums. They are not exposed to
Python directly here — the PyO3 crate maps them to Python exception classes and
string representations in `arcanum-py`.

---

## Step 4 — Lexer (`crates/arcanum-nec-import/src/lexer.rs`)

Implement `parse_deck(input: &str) -> Result<ParsedDeck, ParseError>`:

- Normalize `\r\n` → `\n`
- Skip blank lines and CM/CE lines
- Extract 2-char mnemonic from each non-blank line
- Tokenize remaining content with `split_ascii_whitespace()`
- Parse integer fields with `i32::from_str`, float fields with `f64::from_str`
  (handles scientific notation automatically)
- Hard-error on type mismatch or missing required field, identifying card, line,
  and field position
- Return `NecCard::Unknown(mnemonic)` for unrecognized mnemonics

This is the only place raw text is touched. No regex, no column arithmetic.

---

## Step 5 — Tag registry (`crates/arcanum-nec-import/src/tag_registry.rs`)

`TagRegistry` is a `HashMap<u32, TagEntry>` where `TagEntry` holds wire index,
segment count, and source line number. `insert()` hard-errors on duplicate tag.
`resolve(tag, seg)` hard-errors on unknown tag or out-of-range segment number.
Used exclusively by the router.

---

## Step 6 — Router (`crates/arcanum-nec-import/src/router.rs`)

Implement `route(deck: ParsedDeck) -> Result<(SimulationInput, ParseWarnings), ParseError>`:

- Enforce card ordering (geometry before GE; simulation cards after GE)
- Build tag registry from GW/GA/GH cards; hard-error on duplicate tags
- Record GS scale factor and GM operations verbatim in `GeometryTransforms` (Phase 1 applies them; see `docs/phase1-geometry/design.md` Section 7)
- Split GN: geometric boundary condition → `MeshInput.ground`; electrical
  parameters → `SimulationInput.ground_electrical`
- Assemble frequency list from all FR cards; convert MHz → Hz here
- Validate EX/LD tag and segment references against tag registry
- Emit warnings: unknown cards, unsupported EX types, NRADL > 0, missing EN
- Assemble and return `SimulationInput`

---

## Step 7 — Public Rust API (`crates/arcanum-nec-import/src/lib.rs`)

```rust
pub fn parse(input: &str) -> Result<(SimulationInput, ParseWarnings), ParseError>
pub fn parse_file(path: &Path) -> Result<(SimulationInput, ParseWarnings), ParseError>
```

These are the only public functions of `arcanum-nec-import`. Everything else
(`lexer`, `router`, `tag_registry`) is `pub(crate)`.

---

## Step 8 — PyO3 bindings (`crates/arcanum-py/src/lib.rs`)

The `arcanum-py` crate is the only `cdylib` in the workspace and the only place
`pyo3` is used. It wraps the public APIs of all phase crates.

**Interface style: rich objects.** Each struct that crosses into Python gets a
thin `#[pyclass]` wrapper in `arcanum-py`, exposing fields as `#[getter]`
properties. Python tests use attribute access:

```python
assert sim.mesh_input.wires[0].tag == 1
assert sim.mesh_input.wires[0].segment_count == 11
assert sim.frequencies[0] == pytest.approx(299_792_458.0)
assert sim.sources[0].tag == 1
assert sim.sources[0].segment == 6
```

`ParseError` maps to a Python exception raised on hard parse failure. Warnings
are returned as a list of `PyParseWarning` objects with `.kind`, `.line`, and
`.message` attributes.

For the NEC import phase, the `#[pyclass]` wrappers needed are:
`PySimulationInput`, `PyMeshInput`, `PyWireDescription`, `PyGroundDescriptor`,
`PySourceDefinition`, `PyLoadDefinition`, `PyOutputRequests`, `PyParseWarning`.

---

## Step 9 — Rust unit tests

All V-PARSE, V-FMT, V-ROUTE, V-ERR, and V-WARN cases from `validation.md` are
implemented as `#[test]` functions in `crates/arcanum-nec-import/src/tests/`.
These call `parse()` directly — no Python, no PyO3 overhead. Each test is annotated
with its case ID. This is the fastest feedback loop during development.

Run with: `cargo test -p arcanum-nec-import`

---

## Step 10 — Python integration tests

`tests/nec_import/` contains pytest scripts that import the compiled `arcanum`
extension and exercise the PyO3 boundary.

`tests/nec_import/conftest.py` provides a `reference_deck(name)` fixture loading
`.nec` files from `docs/nec-import/reference-decks/` by filename stem.

`test_real.py` runs V-REAL-001 through V-REAL-004 against the committed reference
decks, asserting on wire counts, segment counts, source tags, and frequencies.

These tests verify that types cross the FFI correctly and that Python exception
handling works as designed — things the Rust tests cannot cover.

Run with: `pytest tests/nec_import/ -v`

---

## Step 11 — GitHub Actions workflow (`.github/workflows/ci.yml`)

```yaml
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: actions/setup-python@v5
        with: { python-version: "3.11" }
      - run: pip install maturin pytest
      - run: cargo test --workspace           # all Rust unit tests across all crates
      - run: maturin develop                   # compile arcanum-py, install into venv
      - run: pytest tests/ -v                 # all Python integration tests

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { components: clippy, rustfmt }
      - run: cargo clippy --workspace -- -D warnings
      - run: cargo fmt --check
```

---

## Implementation order and dependencies

```
Step 1  (workspace scaffold)
  → Step 2  (cards.rs data types)
  → Step 3  (errors.rs)
  → Step 4  (lexer.rs)       ← develop with Step 9 Rust parse + fmt tests
  → Step 5  (tag_registry.rs)
  → Step 6  (router.rs)      ← develop with Step 9 Rust route + err + warn tests
  → Step 7  (lib.rs Rust API)
  → Step 8  (arcanum-py PyO3 bindings)
  → Step 10 (Python integration tests)
  → Step 11 (CI workflow)
```

Phase crates (arcanum-geometry through arcanum-postprocess) are created as empty
stubs in Step 1 so the workspace compiles from day one. They are populated in
future phases.

---

*Arcanum — Open Research Institute*
