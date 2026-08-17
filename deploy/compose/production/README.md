# LawSynth — production Compose stack

A hardened, single-node Docker Compose deployment of LawSynth: API replicas +
scheduler + worker pool + Postgres + object store + NATS, behind a
TLS-terminating reverse proxy. This is the reference self-hosted deployment the
99.9% API-availability SLO is written against.

Use this for a lab or small company on one capable host. Once jobs exceed a
single machine, move to the Kubernetes/Helm deployment under `deploy/helm`.

## Topology

```
Internet ─▶ proxy (Caddy, :443 TLS) ─▶ gateway ─▶ api ─┬─▶ postgres     metadata
                                        (xN)     (xN)  ├─▶ object-store  artifacts
                                                       └─▶ nats          event bus
                                         scheduler (x1) ─▶ postgres, nats
                                         worker (xN)    ─▶ postgres, nats, artifact
```

Only the proxy is published to the host (80/443). Everything else is reachable
only on the internal `lawsynth` network.

## How this differs from `../local`

| Aspect | local | production |
|---|---|---|
| Images | built from source | pulled at a pinned tag |
| Secrets | local-only defaults | required from `.env` |
| Host ports | every service | proxy only |
| TLS | none | Caddy automatic HTTPS |
| Hardening | minimal | resource limits, read-only rootfs, `no-new-privileges`, log rotation |
| Backups | none | `backup.sh` (RPO 15 min target) |

## Files

- `compose.yaml` — orchestrator (`include:` list).
- `postgres.yaml` — metadata database (tuned, checksummed, not published).
- `object-store.yaml` — MinIO + bucket init (swap for managed S3 if desired).
- `nats.yaml` — JetStream event bus with bounded storage.
- `api.yaml` — `api` + `gateway`, replicated, read-only rootfs.
- `worker.yaml` — `artifact` + single `scheduler` + scalable `worker`.
- `proxy.yaml` — Caddy TLS edge, forwards to the gateway.
- `backup.sh` — consistent Postgres + object-store backup with checksums.
- `.env.example` — configuration template; every REQUIRED value must be set.

## Deploy

```bash
cp .env.example .env
# Fill in every REQUIRED value (LAWSYNTH_VERSION, LAWSYNTH_DOMAIN,
# LAWSYNTH_ACME_EMAIL, POSTGRES_PASSWORD, MINIO_ROOT_PASSWORD,
# LAWSYNTH_API_TOKENS, LAWSYNTH_GATEWAY_ALLOWED_ORIGINS).

docker compose --env-file .env config -q     # validate (no daemon needed)
docker login "${LAWSYNTH_REGISTRY}"           # if images are private
docker compose --env-file .env pull
docker compose --env-file .env up -d
docker compose ps
```

DNS for `LAWSYNTH_DOMAIN` must resolve to this host, and ports 80/443 must be
reachable, for Caddy to obtain a certificate. For an internal deployment with
no public ACME, front the stack with your own certificate/terminator and point
it at the gateway on `:8081`.

## Scaling

```bash
docker compose --env-file .env up -d --scale worker=6   # more execution capacity
```

Do **not** scale the scheduler beyond one instance — it must not double-assign
jobs. Scale the API/gateway with `LAWSYNTH_*_REPLICAS` in `.env`.

## Backups

Schedule `backup.sh` (e.g. a cron job or systemd timer every 15 minutes) to
meet the RPO target:

```cron
*/15 * * * * /opt/lawsynth/deploy/compose/production/backup.sh >> /var/log/lawsynth-backup.log 2>&1
```

Each run writes a timestamped set (`metadata.dump`, `objects/`, `manifest.txt`,
`checksums.sha256`) under `BACKUP_ROOT` and prunes to the last `RETENTION` sets.
The Postgres dump and the object mirror are captured together because neither
reconstructs service state alone.

### Restore (RTO target under 2 h)

1. Provision a fresh stack from this directory with a new `.env`.
2. Start only Postgres and the object store:
   `docker compose up -d postgres object-store`.
3. Verify a backup set: `(cd <set> && sha256sum -c checksums.sha256)`.
4. Restore metadata:
   `docker compose exec -T postgres pg_restore --clean --if-exists -U "$POSTGRES_USER" -d "$POSTGRES_DB" < <set>/metadata.dump`.
5. Restore objects: `mc mirror <set>/objects local/$LAWSYNTH_S3_BUCKET`.
6. Bring up the rest: `docker compose up -d`, then confirm `/v1/health` and
   read a known artifact through the API before taking traffic.

## Boundaries

- Point-in-time recovery, cross-region replication, and artifact GC scheduling
  are host-operations concerns, not provided by this stack.
- Studio is served separately (its own image / CDN); publish it under the same
  domain and add its origin to `LAWSYNTH_GATEWAY_ALLOWED_ORIGINS`.
