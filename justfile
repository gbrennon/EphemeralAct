# EphemeralAct — task runner
# Requires: cargo, cargo-llvm-cov, rustfmt, clippy

# Default: show available recipes
default:
    @just --list

# Build the project
build:
    cargo build

# Install debug binary (fast, for local iteration)
install-dev:
    cargo install --path . --debug

# Install release binary to ~/.cargo/bin
install:
    cargo install --path .

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

# Lint Forgejo Actions workflows
lint-workflows:
    actionlint -config-file .actionlint.yaml .forgejo/workflows/*.yml

# Install lefthook pre-commit hooks
install-hooks:
    lefthook install

set shell := ["bash", "-eu", "-o", "pipefail", "-c"]
