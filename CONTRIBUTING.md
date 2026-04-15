# Contributing to Arcanum

## Development Environment Setup

### Required Tools

**Rust toolchain** (stable, 1.75 or later)

Install via rustup: https://rustup.rs

On Windows, rustup also requires the MSVC C++ build tools. Install them via
Visual Studio Installer → Individual Components → "MSVC v143 - VS 2022 C++ x64/x86
build tools" and "Windows 11 SDK". If Visual Studio is already installed for
other purposes, these components may already be present.

After installation, open a new terminal (VS Code must be restarted if it was
already open) and verify:

```
rustc --version
cargo --version
```

**Python 3.9 or later**

Verify: `python --version`

---

### First-Time Setup

Create a virtual environment and install the Python build and test tools into
it. Arcanum's PyO3 extension requires a virtual environment — system-wide
installation will not work with `maturin develop`.

```sh
python -m venv .venv
```

Install dependencies:

```sh
# Windows
.venv\Scripts\pip install maturin pytest

# Linux / macOS
.venv/bin/pip install maturin pytest
```

Build the Rust extension and install it into the virtual environment:

```sh
# Windows
.venv\Scripts\maturin develop

# Linux / macOS
.venv/bin/maturin develop
```

This compiles all Rust crates and produces an importable `arcanum` Python
module in the virtual environment. Re-run this command whenever you change
Rust source code before running Python tests.

---

## Running Tests

### Rust unit tests

```sh
cargo test --workspace
```

Runs 33 unit tests in `arcanum-nec-import` covering V-PARSE, V-FMT, V-ERR,
V-WARN, and V-REAL cases from `docs/nec-import/validation.md`. No separate
build step is needed — `cargo test` compiles and runs in one command.

### Python integration tests

Build the extension first (see above), then:

```sh
# Windows
.venv\Scripts\pytest tests/ -v

# Linux / macOS
.venv/bin/pytest tests/ -v
```

These tests exercise the same validation cases end-to-end through the PyO3
Python bindings. They require the extension to be current with the Rust
source; run `maturin develop` before `pytest` if you have changed any Rust
code since the last build.

### Running both together (matches CI)

```sh
# Linux / macOS
cargo test --workspace && .venv/bin/maturin develop && .venv/bin/pytest tests/ -v

# Windows
cargo test --workspace && .venv\Scripts\maturin develop && .venv\Scripts\pytest tests/ -v
```

---

## Running Linters

### Formatting check

```sh
cargo fmt --check
```

To auto-apply formatting:

```sh
cargo fmt
```

Arcanum uses default rustfmt settings. Aligned columns in match arms and
struct literals are not permitted — rustfmt normalises these automatically.
Run `cargo fmt` before pushing; the CI lint job will fail if formatting
is not clean.

### Clippy

```sh
cargo clippy --workspace -- -D warnings
```

All clippy warnings are treated as errors. The two known suppressions in
`arcanum-py` (`unexpected_cfgs` and `useless_conversion`) are caused by
pyo3 0.22 macro internals and are documented with explanations in that
crate's `Cargo.toml` and `src/lib.rs`.

---

## Continuous Integration

CI runs on every push and pull request to `main` via
`.github/workflows/ci.yml`. Two jobs run in parallel:

| Job | Steps |
|-----|-------|
| **test** | `cargo test --workspace` → `maturin develop` → `pytest tests/ -v` |
| **lint** | `cargo fmt --check` → `cargo clippy --workspace -- -D warnings` |

Both jobs must pass for a PR to be mergeable. PRs require approval from a
reviewer other than the PR author before merging (enforced by branch
protection on `main`).
