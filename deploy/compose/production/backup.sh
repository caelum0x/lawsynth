#!/usr/bin/env bash
#
# Consistent backup of the production LawSynth stack.
#
# Captures the two stores that MUST be backed up together to reconstruct
# service state (see the self-hosting backup guide):
#
#   1. the Postgres metadata database (pg_dump, custom format), and
#   2. the object store bucket (mc mirror), which holds the artifacts the
#      database rows reference by content hash.
#
# A database dump alone, or an object mirror alone, is NOT a restorable backup.
#
# Every backup set is written to a timestamped directory, checksummed, and
# accompanied by a manifest recording the stack revision and settings that
# selected the store locations. Old sets beyond the retention count are pruned.
#
# Reference RPO target: 15 minutes (schedule via cron/systemd timer).
# Reference RTO target: under 2 hours (see README for the restore procedure).
#
# Usage:
#   ./backup.sh                       # write a set under $BACKUP_ROOT
#   BACKUP_ROOT=/mnt/backups ./backup.sh
#   RETENTION=48 ./backup.sh          # keep the last 48 sets
#
# Requires: docker (compose plugin or docker-compose), a running stack, and a
# .env alongside this script.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if [[ -f .env ]]; then
  # shellcheck disable=SC1091
  set -a; . ./.env; set +a
else
  echo "error: .env not found next to backup.sh" >&2
  exit 1
fi

: "${POSTGRES_USER:?POSTGRES_USER must be set in .env}"
: "${POSTGRES_DB:?POSTGRES_DB must be set in .env}"
: "${MINIO_ROOT_USER:?MINIO_ROOT_USER must be set in .env}"
: "${MINIO_ROOT_PASSWORD:?MINIO_ROOT_PASSWORD must be set in .env}"

BACKUP_ROOT="${BACKUP_ROOT:-${SCRIPT_DIR}/backups}"
RETENTION="${RETENTION:-24}"
BUCKET="${LAWSYNTH_S3_BUCKET:-lawsynth-artifacts}"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
SET_DIR="${BACKUP_ROOT}/${STAMP}"

# Prefer the compose plugin, fall back to the standalone binary.
compose() {
  if docker compose version >/dev/null 2>&1; then
    docker compose --env-file "${SCRIPT_DIR}/.env" "$@"
  else
    docker-compose --env-file "${SCRIPT_DIR}/.env" "$@"
  fi
}

log() { printf '[backup %s] %s\n' "$(date -u +%H:%M:%S)" "$*"; }

mkdir -p "${SET_DIR}"
log "writing backup set to ${SET_DIR}"

# --- 1. Postgres metadata database -----------------------------------------
# --format=custom lets us restore selectively with pg_restore; --no-owner keeps
# it portable across role names.
log "dumping Postgres database '${POSTGRES_DB}'"
compose exec -T -e PGPASSWORD="${POSTGRES_PASSWORD:-}" postgres \
  pg_dump --username "${POSTGRES_USER}" --dbname "${POSTGRES_DB}" \
          --format=custom --no-owner --compress=6 \
  > "${SET_DIR}/metadata.dump"

# --- 2. Object store bucket -------------------------------------------------
# Mirror the whole bucket into the set directory. Content addressing means
# re-runs only transfer changed objects.
log "mirroring object bucket '${BUCKET}'"
compose run --rm --no-deps -T \
  -v "${SET_DIR}:/backup" \
  -e MC_HOST_local="http://${MINIO_ROOT_USER}:${MINIO_ROOT_PASSWORD}@object-store:9000" \
  object-store-init \
  /bin/sh -c "mc mirror --overwrite --remove local/${BUCKET} /backup/objects"

# --- 3. Manifest + checksums ------------------------------------------------
log "recording manifest and checksums"
cat > "${SET_DIR}/manifest.txt" <<EOF
lawsynth_backup_version: 1
created_utc: ${STAMP}
postgres_db: ${POSTGRES_DB}
object_bucket: ${BUCKET}
image_version: ${LAWSYNTH_VERSION:-unknown}
registry: ${LAWSYNTH_REGISTRY:-unknown}
host: $(hostname)
EOF

(
  cd "${SET_DIR}"
  # Portable checksum command selection.
  if command -v sha256sum >/dev/null 2>&1; then
    find . -type f ! -name checksums.sha256 -print0 | sort -z \
      | xargs -0 sha256sum > checksums.sha256
  else
    find . -type f ! -name checksums.sha256 -print0 | sort -z \
      | xargs -0 shasum -a 256 > checksums.sha256
  fi
)

# --- 4. Retention -----------------------------------------------------------
log "pruning to the last ${RETENTION} backup sets"
mapfile -t sets < <(find "${BACKUP_ROOT}" -maxdepth 1 -mindepth 1 -type d | sort)
count=${#sets[@]}
if (( count > RETENTION )); then
  remove=$(( count - RETENTION ))
  for (( i = 0; i < remove; i++ )); do
    log "removing old set ${sets[$i]}"
    rm -rf -- "${sets[$i]}"
  done
fi

log "backup complete: ${SET_DIR}"
log "verify with: (cd '${SET_DIR}' && sha256sum -c checksums.sha256)"
