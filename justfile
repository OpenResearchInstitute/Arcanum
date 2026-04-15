# Default recipe — show available commands
default:
    @just --list

# Build and install the Python extension into the venv
develop:
    maturin develop

# Run all Rust unit tests
test-rust:
    cargo test --workspace

# Run all Python integration tests (rebuilds extension first)
test-python: develop
    pytest tests/ -v

# Run the full test suite — Rust then Python
test: test-rust test-python

# Check formatting and clippy
check:
    cargo fmt --check
    cargo clippy --workspace -- -D warnings

# Apply rustfmt
fmt:
    cargo fmt

# Build a release wheel
build:
    maturin build --release
