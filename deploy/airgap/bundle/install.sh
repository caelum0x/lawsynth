#!/usr/bin/env bash
#
# Install and start LawSynth from an air-gapped bundle on the OFFLINE host.
#
# Orchestrates the offline bring-up end to end:
#   1. verify.sh   — integrity-check the bundle,
#   2. import.sh   — load images and stage wheelhouse/datasets,
#   3. render .env — from compose/.env.example if one is not present,
#   4. compose up  — start the production stack against the loaded images.
#
# Because every image is already loaded locally, the compose stack is started
# with --pull never so it never reaches for a registry.
#
# Usage:
#   ./install.sh                          # install the bundle in this dir
#   ./install.sh /path/to/bundle
#   SKIP_VERIFY=1 ./install.sh            # skip integrity check (not advised)
#   NO_START=1 ./install.sh               # prepare everything but don't `up`
#
# You MUST fill in the required secrets in the generated .env before the stack
# will accept traffic. Requires: docker + compose.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUNDLE_DIR="${1:-${SCRIPT_DIR}}"
COMPOSE_DIR="${BUNDLE_DIR}/compose"

log() { printf '[install %s] %s\n' "$(date -u +%H:%M:%S)" "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

command -v docker >/dev/null 2>&1 || die "docker is required"

# Prefer the compose plugin, fall back to the standalone binary.
compose() {
  if docker compose version >/dev/null 2>&1; then
    docker compose "$@"
  else
    docker-compose "$@"
  fi
}

[ -d "${COMPOSE_DIR}" ] || die "compose/ not found in bundle: ${COMPOSE_DIR}"
[ -f "${COMPOSE_DIR}/compose.yaml" ] || die "compose/compose.yaml missing from bundle"

# --- 1. Verify --------------------------------------------------------------
if [ "${SKIP_VERIFY:-0}" != "1" ]; then
  log "verifying bundle integrity"
  bash "${BUNDLE_DIR}/verify.sh" "${BUNDLE_DIR}"
else
  log "SKIP_VERIFY=1 set; skipping integrity check"
fi

# --- 2. Import --------------------------------------------------------------
log "importing images and staging assets"
bash "${BUNDLE_DIR}/import.sh" "${BUNDLE_DIR}"

# --- 3. Environment ---------------------------------------------------------
ENV_FILE="${COMPOSE_DIR}/.env"
if [ ! -f "${ENV_FILE}" ]; then
  if [ -f "${COMPOSE_DIR}/.env.example" ]; then
    cp "${COMPOSE_DIR}/.env.example" "${ENV_FILE}"
    log "created ${ENV_FILE} from template"
    log "ACTION REQUIRED: edit ${ENV_FILE} and set every REQUIRED secret"
  else
    die ".env.example missing from the bundle; cannot render configuration"
  fi
else
  log "using existing ${ENV_FILE}"
fi

# --- 4. Validate + start ----------------------------------------------------
log "validating compose configuration"
compose --env-file "${ENV_FILE}" -f "${COMPOSE_DIR}/compose.yaml" config -q \
  || die "compose configuration invalid — check REQUIRED values in ${ENV_FILE}"

if [ "${NO_START:-0}" = "1" ]; then
  log "NO_START=1 set; prepared but not starting. Start later with:"
  log "  docker compose --env-file '${ENV_FILE}' -f '${COMPOSE_DIR}/compose.yaml' up -d --pull never"
  exit 0
fi

log "starting the stack (offline, --pull never)"
compose --env-file "${ENV_FILE}" -f "${COMPOSE_DIR}/compose.yaml" up -d --pull never

log "stack started"
compose --env-file "${ENV_FILE}" -f "${COMPOSE_DIR}/compose.yaml" ps
log "done. Confirm health once the proxy has a certificate:"
log "  curl -k https://<LAWSYNTH_DOMAIN>/v1/health"
