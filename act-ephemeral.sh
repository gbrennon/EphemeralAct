#!/usr/bin/env bash

set -euo pipefail

# ------------------------------------------------------------------------------
# Configuration
# ------------------------------------------------------------------------------

readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m'

ACT_CONTAINER_ENGINE="${ACT_CONTAINER_ENGINE:-podman}"
CLEANUP_ON_EXIT="${CLEANUP_ON_EXIT:-true}"

# ------------------------------------------------------------------------------
# Usage
# ------------------------------------------------------------------------------

show_usage() {
  cat <<'EOF'
Usage:
  act-ephemeral.sh [OPTIONS] <source_repo_path> [-- ACT_ARGS...]

Run GitHub Actions locally using act from an ephemeral copy of a repository.

OPTIONS:
  -w, --workflow WORKFLOW       Workflow file to run
  -j, --job JOB                 Job ID to run
  -e, --event EVENT             GitHub event name
  -i, --input KEY=VALUE         Workflow input (repeatable)
  -s, --secret KEY=VALUE        Secret (repeatable)
  -c, --container ENGINE        podman|docker (default: podman)
  --no-cleanup                  Preserve temp directory after execution
  -h, --help                    Show this help message

Any arguments after '--' are passed directly to act.

EXAMPLES:

  # Run default workflow
  act-ephemeral.sh /path/to/repo

  # Run a specific workflow
  act-ephemeral.sh \
    -w .github/workflows/ci.yml \
    /path/to/repo

  # Run a specific job
  act-ephemeral.sh \
    -j test \
    /path/to/repo

  # Run with workflow inputs
  act-ephemeral.sh \
    -i environment=staging \
    -i version=1.2.3 \
    /path/to/repo

  # Run with secrets
  act-ephemeral.sh \
    -s GITHUB_TOKEN=xxx \
    -s NPM_TOKEN=yyy \
    /path/to/repo

  # Pass arbitrary act arguments
  act-ephemeral.sh \
    /path/to/repo \
    -- \
    --verbose

  # Use an event payload
  act-ephemeral.sh \
    /path/to/repo \
    -- \
    --eventpath event.json

  # Override runner image
  act-ephemeral.sh \
    /path/to/repo \
    -- \
    -P ubuntu-latest=ghcr.io/catthehacker/ubuntu:act-latest

  # Use a vars file
  act-ephemeral.sh \
    /path/to/repo \
    -- \
    --var-file .vars

  # Enable artifact server
  act-ephemeral.sh \
    /path/to/repo \
    -- \
    --artifact-server-path /tmp/artifacts
EOF
}

# ------------------------------------------------------------------------------
# Helpers
# ------------------------------------------------------------------------------

die() {
  printf "%bError:%b %s\n" "$RED" "$NC" "$*" >&2
  exit 1
}

require_value() {
  local opt="$1"

  if [[ $# -lt 2 || -z "${2:-}" ]]; then
    die "option '$opt' requires a value"
  fi
}

# ------------------------------------------------------------------------------
# Parse arguments
# ------------------------------------------------------------------------------

WORKFLOW=""
JOB=""
EVENT=""
SOURCE_REPO=""

declare -a INPUTS=()
declare -a SECRETS=()
declare -a ACT_EXTRA_ARGS=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --)
      shift
      ACT_EXTRA_ARGS=("$@")
      break
      ;;

    -w|--workflow)
      require_value "$@"
      WORKFLOW="$2"
      shift 2
      ;;

    -j|--job)
      require_value "$@"
      JOB="$2"
      shift 2
      ;;

    -e|--event)
      require_value "$@"
      EVENT="$2"
      shift 2
      ;;

    -i|--input)
      require_value "$@"
      INPUTS+=("$2")
      shift 2
      ;;

    -s|--secret)
      require_value "$@"
      SECRETS+=("$2")
      shift 2
      ;;

    -c|--container)
      require_value "$@"
      ACT_CONTAINER_ENGINE="$2"
      shift 2
      ;;

    --no-cleanup)
      CLEANUP_ON_EXIT="false"
      shift
      ;;

    -h|--help)
      show_usage
      exit 0
      ;;

    -*)
      die "unknown option '$1'"
      ;;

    *)
      if [[ -n "$SOURCE_REPO" ]]; then
        die "multiple repository paths specified"
      fi

      SOURCE_REPO="$1"
      shift
      ;;
  esac
done

# ------------------------------------------------------------------------------
# Validation
# ------------------------------------------------------------------------------

[[ -n "$SOURCE_REPO" ]] || {
  show_usage >&2
  die "source repository path required"
}

[[ -d "$SOURCE_REPO" ]] \
  || die "repository path does not exist: $SOURCE_REPO"

[[ -d "$SOURCE_REPO/.git" || -f "$SOURCE_REPO/.git" ]] \
  || die "'$SOURCE_REPO' is not a git repository"

SOURCE_REPO="$(cd "$SOURCE_REPO" && pwd)"

# ------------------------------------------------------------------------------
# Container socket
# ------------------------------------------------------------------------------

case "$ACT_CONTAINER_ENGINE" in
  podman)
    ACT_CONTAINER_SOCKET="${PODMAN_SOCK:-unix:///run/user/$(id -u)/podman/podman.sock}"
    ;;

  docker)
    ACT_CONTAINER_SOCKET="${DOCKER_SOCK:-unix:///var/run/docker.sock}"
    ;;

  *)
    die "unknown container engine '$ACT_CONTAINER_ENGINE' (expected podman or docker)"
    ;;
esac

# ------------------------------------------------------------------------------
# Ephemeral repository
# ------------------------------------------------------------------------------

REPO_NAME="$(basename "$SOURCE_REPO")"
TEMP_DIR="$(mktemp -d -t "act-run-${REPO_NAME}-XXXXXX")"
readonly TEMP_DIR

cleanup() {
  local rc=$?

  if [[ "$CLEANUP_ON_EXIT" == "true" ]]; then
    if [[ -d "$TEMP_DIR" ]]; then
      printf "%bCleaning up:%b %s\n" \
        "$YELLOW" "$NC" "$TEMP_DIR"

      rm -rf "$TEMP_DIR"
    fi
  else
    printf "%bPreserved temp repository:%b %s\n" \
      "$YELLOW" "$NC" "$TEMP_DIR"
  fi

  exit "$rc"
}

trap cleanup EXIT

printf "%bCreating ephemeral repository:%b %s\n" \
  "$GREEN" "$NC" "$TEMP_DIR"

if command -v rsync >/dev/null 2>&1; then
  rsync -a --delete \
    "$SOURCE_REPO"/ \
    "$TEMP_DIR"/
else
  cp -a "$SOURCE_REPO"/. "$TEMP_DIR"/
fi

# ------------------------------------------------------------------------------
# Ensure standalone git repository
# ------------------------------------------------------------------------------
# Handle git worktrees: when .git is a pointer file instead of a directory,
# convert it to a proper standalone repo so git works inside the container.

if [[ -f "$TEMP_DIR/.git" ]]; then
  printf "%bWorktree detected, converting to standalone repository...%b\n" \
    "$YELLOW" "$NC"

  rm "$TEMP_DIR/.git"
  git -C "$TEMP_DIR" init
  git -C "$TEMP_DIR" add -A
  git -C "$TEMP_DIR" commit -m "ephemeral: snapshot for act" --allow-empty
fi

# ------------------------------------------------------------------------------
# Build act command
# ------------------------------------------------------------------------------

ACT_CMD=(
  act
  --container-daemon-socket "$ACT_CONTAINER_SOCKET"
  --rm
  --bind
)

[[ -n "$WORKFLOW" ]] && ACT_CMD+=(
  -W "$WORKFLOW"
)

[[ -n "$JOB" ]] && ACT_CMD+=(
  -j "$JOB"
)

[[ -n "$EVENT" ]] && ACT_CMD+=(
  "$EVENT"
)

for input in "${INPUTS[@]}"; do
  ACT_CMD+=(
    -i "$input"
  )
done

for secret in "${SECRETS[@]}"; do
  ACT_CMD+=(
    -s "$secret"
  )
done

ACT_CMD+=("${ACT_EXTRA_ARGS[@]}")

# ------------------------------------------------------------------------------
# Execute
# ------------------------------------------------------------------------------

printf "%bWorking directory:%b %s\n" \
  "$GREEN" "$NC" "$TEMP_DIR"

printf "%bRunning:%b\n" \
  "$GREEN" "$NC"

printf ' %q' "${ACT_CMD[@]}"
printf '\n\n'

cd "$TEMP_DIR"

exec "${ACT_CMD[@]}"
