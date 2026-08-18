#!/usr/bin/env bash
# run_all.sh — execute every command shown in the LawSynth guide and assert it
# exits 0. This keeps the guide *executable* and CI-checkable: if a documented
# command breaks, this script fails.
#
# Everything here is deterministic and offline. The datasets are the ones shipped
# next to this script (lotka-volterra.csv, forced-oscillator.csv); no network,
# no RNG seeded from the clock, no external services.
#
# Binary resolution mirrors benchmarks/_engine.py:
#   1. $LAWSYNTH_BIN if set
#   2. <repo>/target/debug/lawsynth
#   3. <repo>/target/release/lawsynth
#   4. fall back to `cargo run --offline -p lawsynth-cli --`
#
# Usage:
#   bash run_all.sh            # build if needed, then run every command
#   LAWSYNTH_BIN=/path/to/lawsynth bash run_all.sh

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# docs/guide/examples -> repo root is three levels up.
ROOT="$(cd "$HERE/../../.." && pwd)"

# --- Locate (or build) the CLI binary --------------------------------------
locate_binary() {
  if [[ -n "${LAWSYNTH_BIN:-}" && -x "${LAWSYNTH_BIN}" ]]; then
    echo "${LAWSYNTH_BIN}"; return 0
  fi
  for candidate in "$ROOT/target/debug/lawsynth" "$ROOT/target/release/lawsynth"; do
    if [[ -x "$candidate" ]]; then echo "$candidate"; return 0; fi
  done
  return 1
}

if BIN="$(locate_binary)"; then
  LAWSYNTH=("$BIN")
else
  echo "no compiled lawsynth binary found; building lawsynth-cli offline..." >&2
  ( cd "$ROOT" && cargo build --offline -p lawsynth-cli >/dev/null )
  if BIN="$(locate_binary)"; then
    LAWSYNTH=("$BIN")
  else
    echo "build succeeded but no binary found; falling back to cargo run" >&2
    LAWSYNTH=(cargo run --offline --quiet -p lawsynth-cli --)
  fi
fi

cd "$HERE"

PASS=0
FAIL=0
run() {
  local label="$1"; shift
  if "${LAWSYNTH[@]}" "$@" >/dev/null 2>&1; then
    printf 'PASS  %s\n' "$label"; PASS=$((PASS + 1))
  else
    printf 'FAIL  %s\n' "$label"; FAIL=$((FAIL + 1))
  fi
}

# --- Regenerate the deterministic forced dataset (byte-identical each run) ---
python3 "$HERE/gen_forced_oscillator.py" >/dev/null

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
LV="$WORK/lotka-volterra.lsworld"

# --- getting-started.md -----------------------------------------------------
run "discover"           discover lotka-volterra.csv --time time --state x,y --preset ecology --output "$LV"
run "inspect"            inspect "$LV"
run "explain"            explain "$LV"
run "simulate"           simulate "$LV" --initial x=10 --initial y=5 --start 0 --end 2 --step 0.5

# --- workflow.md ------------------------------------------------------------
run "simplify"           simplify "$LV"
run "simplify --output"  simplify "$LV" --output "$WORK/lv-simplified.lsworld"
run "stability"          stability "$LV" --box 0:10,0:10
run "stability --json"   stability "$LV" --box 0:10,0:10 --json
run "control"            control forced-oscillator.csv --time time --state x,v --control u --degree 2 --threshold 0.05 --validate
run "export latex"       export "$LV" --format latex
run "export python"      export "$LV" --format python --output "$WORK/lv_model.py"
run "export json"        export "$LV" --format json --output "$WORK/lv.json"
run "validate"           validate "$LV" --data lotka-volterra.csv --time time --holdout 0.2
run "forecast confidence" forecast "$LV" --horizon 5 --start 0 --step 1 --initial x=10 --initial y=5 --confidence --data lotka-volterra.csv --time time --level 0.9 --replicates 100 --seed 7

# --- determinism.md ---------------------------------------------------------
"${LAWSYNTH[@]}" discover lotka-volterra.csv --time time --state x,y --preset ecology --output "$WORK/a.lsworld" >/dev/null 2>&1
"${LAWSYNTH[@]}" discover lotka-volterra.csv --time time --state x,y --preset ecology --output "$WORK/b.lsworld" >/dev/null 2>&1
if cmp -s "$WORK/a.lsworld" "$WORK/b.lsworld"; then
  printf 'PASS  %s\n' "determinism: two discover runs are byte-identical"; PASS=$((PASS + 1))
else
  printf 'FAIL  %s\n' "determinism: two discover runs differ"; FAIL=$((FAIL + 1))
fi

# --- domains round-trip (self-validating presets) ---------------------------
run "domains run damped-oscillator" domains run damped-oscillator
run "domains run lotka-volterra"    domains run lotka-volterra

echo
echo "passed: $PASS   failed: $FAIL"
[[ "$FAIL" -eq 0 ]]
