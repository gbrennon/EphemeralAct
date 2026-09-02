# EphemeralAct — task runner
# Requires: cargo, cargo-llvm-cov, rustfmt, clippy

# Default: show available recipes
default:
	@just --list

# Build the project
build:
	cargo build

# Run the application without installing (pass args through)
# Usage: just run [--help] [--workflow <path>] [--job <name>] [--event <event>] [--input KEY=VALUE] [--secret SECRET] [--extra-arg ARG] [--preserve] [repo-path]
run *args:
	cargo run -- run {{args}}

# Run all workflows found in the repository
run-all-workflows *args:
	cargo run -- run --all-workflows {{args}}

list-workflows:
  cargo run -- list-workflows

list-actions:
  cargo run -- list-actions

# Install debug binary (fast, for local iteration)
install-dev:
	cargo install --path . --debug

# Install release binary to ~/.cargo/bin
install:
	cargo install --path .

# Run all tests with coverage
# Optional: set COVERAGE_THRESHOLD env var (default: 80)
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
	cargo fmt {{ files }}

# Check formatting without modifying files
fmt-check:
	cargo fmt --check

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
