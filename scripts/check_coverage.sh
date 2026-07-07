#!/usr/bin/env bash
set -euo pipefail

# Source common functions
source "$(dirname "$0")/lib/common.sh"

readonly COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-80}"

run_coverage_and_emit_json() {
  echo "Running cargo-llvm-cov (generating JSON report)..."
  cargo llvm-cov --json --output-path cov.json || true
}

run_coverage_and_emit_json
abort_if_coverage_json_is_missing
print_coverage_table
abort_if_line_coverage_is_below_threshold "$COVERAGE_THRESHOLD"