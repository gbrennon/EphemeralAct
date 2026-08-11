#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=scripts/lib/common.sh
source "$(dirname "$0")/lib/common.sh"

readonly COVERAGE_THRESHOLD="${COVERAGE_THRESHOLD:-80}"

container_runtime_available() {
  [ -S /var/run/docker.sock ] && return 0
  [ -S /run/podman/podman.sock ] && return 0
  [ -S "/run/user/$(id -u)/podman/podman.sock" ] && return 0
  return 1
}

run_coverage_and_emit_json() {
  echo "Running cargo-llvm-cov (generating JSON report)..."
  local args=()
  if container_runtime_available; then
    echo "Container runtime detected; including container infrastructure in coverage."
  else
    echo "No container runtime detected; excluding container infrastructure from coverage."
    args+=(--ignore-filename-regex 'src/infrastructure/(runners/.*|container\.rs)')
  fi
  cargo llvm-cov --json --output-path cov.json "${args[@]}" || true
}

run_coverage_and_emit_json
abort_if_coverage_json_is_missing
print_coverage_table
abort_if_line_coverage_is_below_threshold "$COVERAGE_THRESHOLD"
