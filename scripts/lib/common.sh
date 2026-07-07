#!/usr/bin/env bash
# Common shared functions for EphemeralAct scripts
set -euo pipefail

# ----------------------------------------
# Coverage Helpers
# ----------------------------------------

coverage_json_exists() {
  [ -f cov.json ]
}

abort_if_coverage_json_is_missing() {
  if ! coverage_json_exists; then
    echo "ERROR: cov.json not found. cargo-llvm-cov failed to produce JSON output." >&2
    exit 1
  fi
}

extract_coverage_totals_from_json() {
  lines_count=$(jq -r '.data[0].totals.lines.count' cov.json)
  lines_covered=$(jq -r '.data[0].totals.lines.covered' cov.json)
  lines_percent=$(jq -r '.data[0].totals.lines.percent' cov.json)
  functions_percent=$(jq -r '.data[0].totals.functions.percent' cov.json)
  regions_percent=$(jq -r '.data[0].totals.regions.percent' cov.json)

  lines_percent=${lines_percent:-0}
  functions_percent=${functions_percent:-0}
  regions_percent=${regions_percent:-0}

  export lines_count lines_covered lines_percent functions_percent regions_percent
}

normalize_path() {
  local path="$1"
  path="${path##*/src/}"
  echo "src/${path}"
}

extract_missing_lines() {
  local file_path="$1"
  # Segments schema: [line, col, count, has_count, is_region_entry, is_gap_region]
  # A segment with count=0 and has_count=false starts an uncovered region.
  # The next segment (any count) closes it.
  jq -r --arg fp "$file_path" '
    .data[0].files[]
    | select(.filename == $fp)
    | .segments as $segs
    | [ range(0; $segs | length)
        | . as $i
        | $segs[$i]
        | select(.[2] == 0 and .[3] == false)
        | { start: .[0], end: ($segs[$i+1] // .[0:1])[0] }
      ]
    | group_by(.start)
    | map(.[0])
    | map(
        if .start == .end then (.start | tostring)
        else "\(.start)-\(.end)"
        end
      )
    | join(", ")
  ' cov.json
}

print_coverage_table() {
  extract_coverage_totals_from_json

  printf "\n"

  # Collect normalized rows into a temp file so we can measure column widths first
  local tmp_rows
  tmp_rows=$(mktemp)

  jq -r '
    .data[0].files[]
    | select(.summary.lines.count > 0)
    | [
        .filename,
        (.summary.lines.count | tostring),
        ((.summary.lines.count - .summary.lines.covered) | tostring),
        .summary.lines.percent
      ]
    | @tsv
  ' cov.json | while IFS=$'\t' read -r raw_path stmts miss pct; do
    local norm
    norm=$(normalize_path "$raw_path")
    printf '%s\t%s\t%s\t%s\t%s\n' "$norm" "$stmts" "$miss" "$pct" "$raw_path"
  done | sort > "$tmp_rows"

  # Compute column widths
  local max_name_len
  max_name_len=$(awk -F'\t' '{print length($1)}' "$tmp_rows" | sort -n | tail -1)
  local name_col=$(( max_name_len > 4 ? max_name_len : 4 ))

  local max_missing_len=7  # minimum width for "Missing" header
  while IFS=$'\t' read -r norm stmts miss pct raw_path; do
    if [ "$miss" -gt 0 ]; then
      local mlines
      mlines=$(extract_missing_lines "$raw_path")
      local mlen=${#mlines}
      [ "$mlen" -gt "$max_missing_len" ] && max_missing_len=$mlen
    fi
  done < "$tmp_rows"

  # Limit max_missing_len to prevent excessive width
  if [ "$max_missing_len" -gt 80 ]; then
    max_missing_len=80
  fi

  local sep
  sep=$(printf '%*s' $(( name_col + 28 + max_missing_len )) '' | tr ' ' '-')

  printf "%-${name_col}s  %6s  %4s  %6s  %-${max_missing_len}s\n" \
    "Name" "Stmts" "Miss" "Cover" "Missing"
  echo "$sep"

  while IFS=$'\t' read -r norm stmts miss pct raw_path; do
    local missing_lines=""
    if [ "$miss" -gt 0 ]; then
      missing_lines=$(extract_missing_lines "$raw_path")
      # Truncate very long missing line lists to prevent table breakage
      if [ "${#missing_lines}" -gt "$max_missing_len" ]; then
        missing_lines="${missing_lines:0:$((max_missing_len-4))} ..."
      fi
    fi
    printf "%-${name_col}s  %6s  %4s  %5.1f%%  %-${max_missing_len}s\n" \
      "$norm" "$stmts" "$miss" "$pct" "$missing_lines"
  done < "$tmp_rows"

  echo "$sep"
  printf "%-${name_col}s  %6s  %4s  %5.1f%%\n" \
    "TOTAL" "$lines_count" "$(( lines_count - lines_covered ))" "$lines_percent"

  printf "\n"
  printf "  Functions: %.1f%%\n" "$functions_percent"
  printf "  Regions:   %.1f%%\n" "$regions_percent"

  rm -f "$tmp_rows"
}

abort_if_line_coverage_is_below_threshold() {
  extract_coverage_totals_from_json
  local threshold="$1"
  local passes
  passes=$(awk -v p="$lines_percent" -v t="$threshold" \
    'BEGIN{ if (p+0 >= t+0) print 1; else print 0 }')
  if [ "$passes" -eq 1 ]; then
    printf "Coverage check: PASS (>= %s%%)\n" "$threshold"
  else
    printf "Coverage check: FAIL (< %s%%)\n" "$threshold"
    exit 1
  fi
}