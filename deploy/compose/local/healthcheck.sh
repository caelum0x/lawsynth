#!/usr/bin/env bash
#
# Poll the local LawSynth stack until every service reports healthy, or fail
# after a timeout. Intended to be run after `docker compose up -d`.
#
#   ./healthcheck.sh              # use defaults / values from .env
#   TIMEOUT=180 ./healthcheck.sh  # wait longer for first-time image builds
#
# Exit codes: 0 all healthy, 1 timed out, 2 a container exited/unhealthy.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Load .env if present so host ports match the running stack.
if [[ -f .env ]]; then
  # shellcheck disable=SC1091
  set -a; . ./.env; set +a
fi

TIMEOUT="${TIMEOUT:-120}"
INTERVAL="${INTERVAL:-3}"

GATEWAY_PORT="${LAWSYNTH_GATEWAY_HOST_PORT:-8081}"
API_PORT="${LAWSYNTH_API_HOST_PORT:-8080}"
ARTIFACT_PORT="${LAWSYNTH_ARTIFACT_HOST_PORT:-8082}"
NATS_PORT="${NATS_MONITOR_HOST_PORT:-8222}"
MINIO_PORT="${MINIO_API_HOST_PORT:-9000}"

# Prefer the compose plugin, fall back to the standalone binary.
compose() {
  if docker compose version >/dev/null 2>&1; then
    docker compose "$@"
  else
    docker-compose "$@"
  fi
}

# name|url pairs. Empty url => check container health only (no HTTP surface).
CHECKS=(
  "gateway|http://127.0.0.1:${GATEWAY_PORT}/healthz"
  "api|http://127.0.0.1:${API_PORT}/v1/health"
  "artifact|http://127.0.0.1:${ARTIFACT_PORT}/health"
  "nats|http://127.0.0.1:${NATS_PORT}/healthz"
  "minio|http://127.0.0.1:${MINIO_PORT}/minio/health/ready"
  "postgres|"
  "scheduler|"
  "worker|"
)

http_ok() {
  curl --fail --silent --show-error --max-time 5 --output /dev/null "$1"
}

container_healthy() {
  local svc="$1" cid state health
  cid="$(compose ps -q "$svc" 2>/dev/null || true)"
  [[ -n "$cid" ]] || return 1
  state="$(docker inspect -f '{{.State.Status}}' "$cid" 2>/dev/null || echo unknown)"
  [[ "$state" == "exited" || "$state" == "dead" ]] && return 2
  health="$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$cid" 2>/dev/null || echo none)"
  case "$health" in
    healthy|none) return 0 ;;
    *) return 1 ;;
  esac
}

echo "Waiting up to ${TIMEOUT}s for the local LawSynth stack to become healthy..."
deadline=$(( $(date +%s) + TIMEOUT ))

while :; do
  pending=()
  fatal=0
  for entry in "${CHECKS[@]}"; do
    svc="${entry%%|*}"
    url="${entry#*|}"

    if ! container_healthy "$svc"; then
      rc=$?
      if [[ $rc -eq 2 ]]; then
        echo "FATAL: container '$svc' has exited." >&2
        fatal=1
      fi
      pending+=("$svc")
      continue
    fi

    if [[ -n "$url" ]] && ! http_ok "$url"; then
      pending+=("$svc")
    fi
  done

  if [[ $fatal -eq 1 ]]; then
    compose ps
    exit 2
  fi

  if [[ ${#pending[@]} -eq 0 ]]; then
    echo "All services healthy."
    exit 0
  fi

  if [[ $(date +%s) -ge $deadline ]]; then
    echo "Timed out. Still not ready: ${pending[*]}" >&2
    compose ps
    exit 1
  fi

  echo "  not ready yet: ${pending[*]}"
  sleep "$INTERVAL"
done
