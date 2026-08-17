#!/usr/bin/env bash
# Install the LawSynth single-node stack under systemd.
#
# Idempotent: safe to re-run for upgrades. It creates the service user, lays out
# the directory tree, installs unit files, and enables the aggregate target. It
# does NOT install the service binaries/venv or provision Postgres/NATS -- those
# are prerequisites (see README.md). It also does not overwrite an existing
# /etc/lawsynth/environment so your secrets are never clobbered.
set -euo pipefail

SERVICE_USER="lawsynth"
SERVICE_GROUP="lawsynth"
PREFIX="/opt/lawsynth"
STATE_DIR="/var/lib/lawsynth"
CONFIG_DIR="/etc/lawsynth"
UNIT_DIR="/etc/systemd/system"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

UNITS=(
  lawsynth-artifact.service
  lawsynth-scheduler.service
  lawsynth-api.service
  lawsynth-gateway.service
  lawsynth-worker.service
  lawsynth.target
)

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "error: must run as root (try: sudo $0)" >&2
    exit 1
  fi
}

ensure_user() {
  if ! getent group "${SERVICE_GROUP}" >/dev/null; then
    groupadd --system "${SERVICE_GROUP}"
  fi
  if ! getent passwd "${SERVICE_USER}" >/dev/null; then
    useradd --system --gid "${SERVICE_GROUP}" \
      --home-dir "${STATE_DIR}" --no-create-home \
      --shell /usr/sbin/nologin "${SERVICE_USER}"
  fi
}

ensure_dirs() {
  install -d -o root -g root -m 0755 "${PREFIX}" "${PREFIX}/bin"
  install -d -o "${SERVICE_USER}" -g "${SERVICE_GROUP}" -m 0750 "${STATE_DIR}"
  install -d -o "${SERVICE_USER}" -g "${SERVICE_GROUP}" -m 0750 \
    "${STATE_DIR}/objects" "${STATE_DIR}/cache" "${STATE_DIR}/work"
  install -d -o root -g "${SERVICE_GROUP}" -m 0750 "${CONFIG_DIR}"
}

ensure_environment() {
  local target="${CONFIG_DIR}/environment"
  if [[ -f "${target}" ]]; then
    echo "keeping existing ${target}"
  else
    install -o root -g "${SERVICE_GROUP}" -m 0640 \
      "${SRC_DIR}/environment.example" "${target}"
    echo "installed ${target} from template -- EDIT IT before starting the stack"
  fi
}

install_units() {
  local unit
  for unit in "${UNITS[@]}"; do
    install -o root -g root -m 0644 "${SRC_DIR}/${unit}" "${UNIT_DIR}/${unit}"
    echo "installed ${UNIT_DIR}/${unit}"
  done
}

main() {
  require_root
  ensure_user
  ensure_dirs
  ensure_environment
  install_units
  systemctl daemon-reload
  systemctl enable lawsynth.target
  cat <<'EOF'

LawSynth units installed and enabled.

Next steps:
  1. Place binaries:   /opt/lawsynth/bin/{lawsynth-artifact,lawsynth-scheduler,lawsynth-worker}
  2. Place Python env: /opt/lawsynth/venv  (with lawsynth_api and lawsynth_gateway installed)
  3. Edit secrets:     /etc/lawsynth/environment
  4. Ensure Postgres and NATS are running.
  5. Start the stack:  systemctl start lawsynth.target
  6. Check status:     systemctl status 'lawsynth-*'
EOF
}

main "$@"
