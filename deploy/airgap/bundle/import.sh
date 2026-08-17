#!/usr/bin/env bash
#
# Import a LawSynth air-gapped bundle on the OFFLINE target host.
#
# Loads the saved container images into the local docker engine and stages the
# Python wheelhouse and datasets into their target directories. Idempotent:
# re-running loads the same images again (a no-op) and re-syncs staged files.
#
# Run verify.sh BEFORE this script. install.sh calls both for you.
#
# Usage:
#   ./import.sh                 # import the bundle in this directory
#   ./import.sh /path/to/bundle
#
# Overridable staging locations:
#   WHEELHOUSE_DIR (default /opt/lawsynth/wheelhouse)
#   DATASET_ROOT   (default /var/lib/lawsynth/datasets)
#
# Requires: docker.

set -euo pipefail

BUNDLE_DIR="${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)}"
WHEELHOUSE_DIR="${WHEELHOUSE_DIR:-/opt/lawsynth/wheelhouse}"
DATASET_ROOT="${DATASET_ROOT:-/var/lib/lawsynth/datasets}"

log() { printf '[import %s] %s\n' "$(date -u +%H:%M:%S)" "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || die "docker is required"
[ -d "${BUNDLE_DIR}" ] || die "bundle directory not found: ${BUNDLE_DIR}"

# --- 1. Container images ----------------------------------------------------
if [ -d "${BUNDLE_DIR}/images" ]; then
  log "loading container images"
  loaded=0
  for tar in "${BUNDLE_DIR}"/images/*.tar; do
    [ -e "$tar" ] || { log "  no image tarballs found"; break; }
    log "  load: $(basename "$tar")"
    docker load -i "$tar"
    loaded=$((loaded + 1))
  done
  log "loaded ${loaded} image(s)"
else
  log "no images/ directory; skipping image load"
fi

# --- 2. Python wheelhouse ---------------------------------------------------
if [ -d "${BUNDLE_DIR}/wheels" ] && compgen -G "${BUNDLE_DIR}/wheels/*.whl" >/dev/null; then
  log "staging wheelhouse into ${WHEELHOUSE_DIR}"
  mkdir -p "${WHEELHOUSE_DIR}"
  cp -f "${BUNDLE_DIR}"/wheels/*.whl "${WHEELHOUSE_DIR}/"
  log "  install SDK/API offline with:"
  log "    pip install --no-index --find-links '${WHEELHOUSE_DIR}' lawsynth-api"
else
  log "no wheels to stage"
fi

# --- 3. Datasets ------------------------------------------------------------
if [ -d "${BUNDLE_DIR}/datasets" ]; then
  log "staging datasets into ${DATASET_ROOT}"
  mkdir -p "${DATASET_ROOT}"
  # Copy the tree preserving relative layout.
  ( cd "${BUNDLE_DIR}/datasets" && find . -type f -print0 ) \
    | while IFS= read -r -d '' rel; do
        dest="${DATASET_ROOT}/${rel#./}"
        mkdir -p "$(dirname "$dest")"
        cp -f "${BUNDLE_DIR}/datasets/${rel#./}" "$dest"
      done
else
  log "no datasets/ directory; skipping"
fi

log "import complete"
log "next: run install.sh to render .env and start the stack"
