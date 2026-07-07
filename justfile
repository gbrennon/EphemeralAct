# EphemeralAct — task runner
# Requires: cargo, cargo-llvm-cov, rustfmt, clippy

# Default: check formatting, lint, then run tests with coverage
default: fmt-check lint test

# Build the project
build:
    cargo build

# Run all tests with coverage
test:
    COVERAGE_THRESHOLD=80 ./scripts/check_coverage.sh

# Run tests without coverage (faster, for local development)
test-local:
    cargo test

# Lint (zero warnings enforced)
lint:
    cargo clippy -- -D warnings

# Lint fixes (optionally specify files)
lint-fix +files='':
    cargo clippy --fix --allow-dirty --allow-staged {{files}}

# Format source files (optionally specify files)
fmt *files:
    cargo +nightly fmt {{ files }}

# Check formatting without modifying files
fmt-check:
    cargo +nightly fmt --check

# Install required dev tools
tools:
    rustup component add rustfmt clippy
    cargo install cargo-llvm-cov --locked --force

# Remove build artifacts
clean:
    cargo clean

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]