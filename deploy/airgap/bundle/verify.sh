#!/usr/bin/env bash
#
# Verify the integrity of a LawSynth air-gapped bundle.
#
# Recomputes sha256 for every file recorded in checksums.sha256 and compares.
# Comment/blank lines in checksums.sha256 are ignored, so the committed
# placeholder verifies as "empty (not yet exported)" instead of failing.
#
# Usage:
#   ./verify.sh              # verify the bundle in this directory
#   ./verify.sh /path/to/bundle
#
# Exit codes: 0 verified (or empty placeholder), 1 mismatch/missing files,
# 2 usage/environment error.

set -euo pipefail

BUNDLE_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
CHECKSUMS="${BUNDLE_DIR}/checksums.sha256"

die() { printf 'error: %s\n' "$*" >&2; exit 2; }

[ -f "${CHECKSUMS}" ] || die "checksums.sha256 not found in ${BUNDLE_DIR}"

# Strip comment and blank lines into a temp file with the real entries only.
work="$(mktemp)"
trap 'rm -f "${work}"' EXIT
grep -vE '^\s*(#|$)' "${CHECKSUMS}" > "${work}" || true

if [ ! -s "${work}" ]; then
  echo "checksums.sha256 has no entries — this looks like an un-exported"
  echo "placeholder bundle. Run export.sh on a connected host first."
  exit 0
fi

entries="$(wc -l < "${work}" | tr -d ' ')"
echo "Verifying ${entries} file(s) in ${BUNDLE_DIR} ..."

# Pick a checker.
if command -v sha256sum >/dev/null 2>&1; then
  checker=(sha256sum -c --strict)
elif command -v shasum >/dev/null 2>&1; then
  checker=(shasum -a 256 -c)
else
  die "no sha256 tool found (need sha256sum or shasum)"
fi

# Run the check from inside the bundle so the recorded relative paths resolve.
if ( cd "${BUNDLE_DIR}" && "${checker[@]}" "${work}" ); then
  echo "OK: all files match their recorded checksums."
  exit 0
else
  echo "FAILED: one or more files are missing or corrupt." >&2
  exit 1
fi
