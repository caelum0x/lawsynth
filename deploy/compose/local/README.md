# LawSynth — local Compose stack

A one-machine Docker Compose stack that runs the full LawSynth service graph
for development and evaluation. This is the **Single-node server** deployment
mode from the production architecture: API + worker + Postgres + MinIO, plus
the scheduler, admission gateway, and content-addressed artifact service.

> This stack ships **local-only default credentials** and terminates no TLS.
> Use it on a trusted developer machine only. For a hardened, TLS-terminated
> deployment use [`../production`](../production/).

## Topology

```
studio ─▶ gateway :8081 ─▶ api :8080 ─┬─▶ postgres :5432   metadata database
                                      ├─▶ minio :9000      S3 object store
                                      └─▶ nats :4222       event bus
scheduler ─▶ postgres, nats
worker    ─▶ postgres, nats, artifact :8082
```

| Service | Image | Host port | Purpose |
|---|---|---:|---|
| gateway | `gateway` | 8081 | Admission (limits, rate window, CORS) in front of the API |
| api | `api` | 8080 | WSGI facade over the domain core |
| artifact | `artifact` | 8082 | Content-addressed artifact lifecycle |
| scheduler | `scheduler` | — | Job assignment, leasing |
| worker | `worker` | — | Job execution (scale with `--scale worker=N`) |
| postgres | `postgres:16-alpine` | 5432 | Metadata database |
| minio | `minio` | 9000 / 9001 | Object store + console |
| nats | `nats:2.10-alpine` | 4222 / 8222 | JetStream event bus + monitoring |

## Files

The stack is assembled from small, single-purpose files merged by
`compose.yaml`'s `include:` directive:

- `compose.yaml` — orchestrator; lists the includes.
- `postgres.yaml`, `minio.yaml`, `nats.yaml` — backing infrastructure.
- `api.yaml` — `api` + `gateway`.
- `worker.yaml` — `artifact` + `scheduler` + `worker`.
- `volumes.yaml` — canonical shared network + persistent volume declarations.
- `.env.example` — configuration template (copy to `.env`).
- `healthcheck.sh` — waits for the whole stack to report healthy.

## Quick start

```bash
cp .env.example .env          # edit if you want non-default ports/credentials
docker compose up --build -d  # first run builds all images from the repo root
./healthcheck.sh              # blocks until every service is healthy
```

Then:

```bash
# Health through the gateway (what Studio / the CLI talk to):
curl -H 'Authorization: Bearer local-dev-token' http://localhost:8081/v1/health

# Object-store console: http://localhost:9001  (user/pass from .env)
# NATS monitoring:      http://localhost:8222/
```

## Operations

```bash
docker compose ps                       # status
docker compose logs -f api worker       # follow logs
docker compose up -d --scale worker=3   # more execution capacity
docker compose restart gateway          # bounce one service
docker compose down                     # stop (keeps volumes/data)
docker compose down --volumes           # stop AND delete all data
```

Configuration is validated (no daemon required) with:

```bash
docker compose -f compose.yaml config -q
```

## Persistent state

Four named volumes hold all state — see `volumes.yaml`:
`lawsynth_pgdata`, `lawsynth_minio`, `lawsynth_nats`, `lawsynth_artifacts`.
Remove them with `docker compose down --volumes` to get a clean slate.

## Notes and boundaries

- Studio is not started here; run it from the pnpm workspace (`apps/studio`)
  and point it at `http://localhost:8081`, or add the `studio` image yourself.
- The gateway image bundles the API package and wraps it in the admission
  layer; that is why the gateway also receives database/object-store settings.
- TLS, real secret management, backups, and rate-limit tuning belong to the
  production overlay, not this stack.
