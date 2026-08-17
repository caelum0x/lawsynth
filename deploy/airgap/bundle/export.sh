#!/usr/bin/env bash
#
# Build a LawSynth air-gapped bundle on a CONNECTED host.
#
# Collects everything an offline host needs to run the production stack:
#   - container images   (docker save,  driven by images.txt)
#   - a Python wheelhouse (pip download / pip wheel, driven by packages.txt)
#   - reference datasets  (copied or downloaded, driven by datasets.txt)
#   - the production compose profile (copied verbatim)
#   - a stamped manifest.yaml and a checksums.sha256 over the whole bundle
#
# Usage:
#   ./export.sh                       # -> ./dist/lawsynth-airgap-<version>/
#   BUNDLE_VERSION=0.1.0 ./export.sh
#   OUTPUT_DIR=/mnt/usb ./export.sh
#   ARCHIVE=1 ./export.sh             # also produce a single .tar.gz
#
# Requires: docker, pip (python3 -m pip), tar, and network access.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# deploy/airgap/bundle -> repository root is three levels up.
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"

BUNDLE_VERSION="${BUNDLE_VERSION:-0.1.0}"
TARGET_PLATFORM="${TARGET_PLATFORM:-linux/amd64}"
OUTPUT_DIR="${OUTPUT_DIR:-${SCRIPT_DIR}/dist}"
BUNDLE_NAME="lawsynth-airgap-${BUNDLE_VERSION}"
BUNDLE_DIR="${OUTPUT_DIR}/${BUNDLE_NAME}"

log() { printf '[export %s] %s\n' "$(date -u +%H:%M:%S)" "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

# Emit the non-comment, non-blank lines of a list file.
read_list() { grep -vE '^\s*(#|$)' "$1" || true; }

# A portable sha256 checksum writer: `checksum_dir <dir>` writes checksums.sha256
# in <dir> covering every file under it except that file.
checksum_dir() {
  local dir="$1"
  ( cd "$dir"
    if command -v sha256sum >/dev/null 2>&1; then
      find . -type f ! -name checksums.sha256 -print0 | LC_ALL=C sort -z \
        | xargs -0 sha256sum
    else
      find . -type f ! -name checksums.sha256 -print0 | LC_ALL=C sort -z \
        | xargs -0 shasum -a 256
    fi
  ) > "${dir}/checksums.sha256"
}

command -v docker >/dev/null 2>&1 || die "docker is required"
command -v tar    >/dev/null 2>&1 || die "tar is required"

log "building ${BUNDLE_NAME} for ${TARGET_PLATFORM}"
rm -rf "${BUNDLE_DIR}"
mkdir -p "${BUNDLE_DIR}/images" "${BUNDLE_DIR}/wheels" \
         "${BUNDLE_DIR}/datasets" "${BUNDLE_DIR}/compose"

# --- 1. Container images ----------------------------------------------------
log "pulling and saving container images"
while IFS= read -r image; do
  [ -n "$image" ] || continue
  log "  image: ${image}"
  docker pull --platform "${TARGET_PLATFORM}" "$image"
  # Sanitize the reference into a flat filename.
  safe="$(printf '%s' "$image" | tr '/:@' '___')"
  docker save "$image" -o "${BUNDLE_DIR}/images/${safe}.tar"
done < <(read_list "${SCRIPT_DIR}/images.txt")

# --- 2. Python wheelhouse ---------------------------------------------------
log "assembling Python wheelhouse"
# Local LawSynth distributions are built from source (never fetched remotely).
for pkg in python/lawsynth-server services/api services/gateway; do
  if [ -d "${REPO_ROOT}/${pkg}" ]; then
    log "  wheel: ${pkg}"
    python3 -m pip wheel --no-deps --wheel-dir "${BUNDLE_DIR}/wheels" "${REPO_ROOT}/${pkg}"
  fi
done
# Third-party requirements (skip local dists, which are handled above).
tmp_reqs="$(mktemp)"
trap 'rm -f "${tmp_reqs}"' EXIT
read_list "${SCRIPT_DIR}/packages.txt" | grep -viE '^lawsynth-' > "${tmp_reqs}" || true
if [ -s "${tmp_reqs}" ]; then
  python3 -m pip download --dest "${BUNDLE_DIR}/wheels" --requirement "${tmp_reqs}"
fi

# --- 3. Datasets ------------------------------------------------------------
log "collecting datasets"
while IFS= read -r line; do
  [ -n "$line" ] || continue
  dest="$(printf '%s' "$line" | awk '{print $1}')"
  src="$(printf '%s' "$line"  | awk '{print $2}')"
  [ -n "$dest" ] && [ -n "$src" ] || { log "  skip malformed dataset line: $line"; continue; }
  out="${BUNDLE_DIR}/datasets/${dest}"
  mkdir -p "$(dirname "$out")"
  case "$src" in
    repo:*)
      rel="${src#repo:}"
      if [ -f "${REPO_ROOT}/${rel}" ]; then
        log "  copy: ${rel}"
        cp "${REPO_ROOT}/${rel}" "$out"
      else
        log "  WARN: repo dataset missing, skipping: ${rel}"
      fi
      ;;
    http://*|https://*)
      log "  fetch: ${src}"
      curl -fsSL "$src" -o "$out"
      ;;
    *)
      log "  WARN: unknown dataset source scheme, skipping: ${src}"
      ;;
  esac
done < <(read_list "${SCRIPT_DIR}/datasets.txt")

# --- 4. Compose profile + list files ---------------------------------------
log "copying production compose profile"
cp "${REPO_ROOT}/deploy/compose/production/"* "${BUNDLE_DIR}/compose/" 2>/dev/null || true
cp "${SCRIPT_DIR}/images.txt" "${SCRIPT_DIR}/packages.txt" \
   "${SCRIPT_DIR}/datasets.txt" "${BUNDLE_DIR}/"

# --- 5. Stamped manifest ----------------------------------------------------
log "writing manifest"
created="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
sed \
  -e "s|bundleVersion: \".*\"|bundleVersion: \"${BUNDLE_VERSION}\"|" \
  -e "s|createdUtc: \".*\"|createdUtc: \"${created}\"|" \
  -e "s|targetPlatform: .*|targetPlatform: ${TARGET_PLATFORM}|" \
  "${SCRIPT_DIR}/manifest.yaml" > "${BUNDLE_DIR}/manifest.yaml"

# --- 6. Checksums -----------------------------------------------------------
log "computing checksums"
checksum_dir "${BUNDLE_DIR}"

log "bundle ready: ${BUNDLE_DIR}"

# --- 7. Optional single archive --------------------------------------------
if [ "${ARCHIVE:-0}" = "1" ]; then
  log "creating archive"
  ( cd "${OUTPUT_DIR}" && tar -czf "${BUNDLE_NAME}.tar.gz" "${BUNDLE_NAME}" )
  log "archive ready: ${OUTPUT_DIR}/${BUNDLE_NAME}.tar.gz"
fi

log "next: transfer the bundle, then run verify.sh and install.sh on the target"
