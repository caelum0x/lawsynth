#!/usr/bin/env bash
# Remove the LawSynth single-node systemd stack.
#
# Stops and disables the units and deletes the unit files. By default it
# PRESERVES data and secrets (/var/lib/lawsynth, /etc/lawsynth) and the service
# user. Pass --purge to also remove those -- this is destructive and irreversible.
set -euo pipefail

SERVICE_USER="lawsynth"
SERVICE_GROUP="lawsynth"
PREFIX="/opt/lawsynth"
STATE_DIR="/var/lib/lawsynth"
CONFIG_DIR="/etc/lawsynth"
UNIT_DIR="/etc/systemd/system"

UNITS=(
  lawsynth-gateway.service
  lawsynth-api.service
  lawsynth-worker.service
  lawsynth-scheduler.service
  lawsynth-artifact.service
  lawsynth.target
)

PURGE=0
if [[ "${1:-}" == "--purge" ]]; then
  PURGE=1
fi

require_root() {
  if [[ "${EUID}" -ne 0 ]]; then
    echo "error: must run as root (try: sudo $0)" >&2
    exit 1
  fi
}

stop_units() {
  systemctl stop lawsynth.target 2>/dev/null || true
  local unit
  for unit in "${UNITS[@]}"; do
    systemctl disable "${unit}" 2>/dev/null || true
    systemctl stop "${unit}" 2>/dev/null || true
    rm -f "${UNIT_DIR}/${unit}"
  done
  systemctl daemon-reload
  systemctl reset-failed 2>/dev/null || true
}

purge_data() {
  echo "purging data, secrets, binaries, and service user"
  rm -rf "${PREFIX}" "${STATE_DIR}" "${CONFIG_DIR}"
  if getent passwd "${SERVICE_USER}" >/dev/null; then
    userdel "${SERVICE_USER}" 2>/dev/null || true
  fi
  if getent group "${SERVICE_GROUP}" >/dev/null; then
    groupdel "${SERVICE_GROUP}" 2>/dev/null || true
  fi
}

main() {
  require_root
  stop_units
  echo "units stopped, disabled, and removed"
  if [[ "${PURGE}" -eq 1 ]]; then
    purge_data
  else
    echo "preserved: ${STATE_DIR}, ${CONFIG_DIR}, ${PREFIX}, and user ${SERVICE_USER}"
    echo "re-run with --purge to remove them."
  fi
}

main "$@"
