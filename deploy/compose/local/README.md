# LawSynth — local Compose stack

A one-machine Docker Compose stack for development and evaluation. This is the
**Single-node server** deployment mode from the production architecture.

> This stack ships **local-only default credentials** and terminates no TLS.
> Use it on a trusted developer machine only. For a hardened, TLS-terminated
> deployment use [`../production`](../production/).
>
> **Just the discovery loop?** For the minimal, self-contained API + gateway
> profile (no Postgres/MinIO/NATS) see [`../../quickstart`](../../quickstart/)
> and the [self-hosting quickstart](../../../docs/self-hosting/quickstart.md).

## Topology

The stack has two planes. The **product loop** is runnable today; the **Rust
execution plane** is present as infrastructure but is not yet wired to the API's
in-process discovery.

```
Product loop (runnable):
  client ─▶ gateway :8081 ─▶ api :8080 ─┬─▶ SQLite  metadata     (api data volume)
                                        └─▶ files   object store  (api data volume)

Rust execution plane (not wired to the API yet):
  scheduler ─▶ postgres :5432, nats :4222
  worker    ─▶ postgres, nats, artifact :8082
```

The API runs discovery on the compiled `lawsynth` engine **in-process**; it uses
**SQLite + filesystem**, not Postgres/MinIO/NATS. Those services back the Rust
plane only.

| Service | Image | Host port | Plane | Purpose |
|---|---|---:|---|---|
| gateway | `gateway` | 8081 | product | Admission (limits, rate window, CORS) in front of the API |
| api | `api` | 8080 | product | WSGI facade over the domain core (SQLite + files) |
| artifact | `artifact` | 8082 | rust | Content-addressed artifact lifecycle |
| scheduler | `scheduler` | — | rust | Job assignment, leasing |
| worker | `worker` | — | rust | Job execution (scale with `--scale worker=N`) |
| postgres | `postgres:16-alpine` | 5432 | rust | Metadata database (Rust plane) |
| minio | `minio` | 9000 / 9001 | rust | Object store + console (Rust plane) |
| nats | `nats:2.10-alpine` | 4222 / 8222 | rust | JetStream event bus + monitoring (Rust plane) |

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
# Gateway liveness (its own /healthz — no auth, not proxied to the API):
curl http://localhost:8081/healthz

# API readiness through the gateway (proxied; /v1/health needs no auth):
curl http://localhost:8081/v1/health         # {"status":"ok","database":"ok","storage":"ok"}

# Then follow the discovery walkthrough (submit a run, poll, fetch the report):
#   ../../../docs/self-hosting/quickstart.md

# Object-store console: http://localhost:9001  (user/pass from .env) — Rust plane
# NATS monitoring:      http://localhost:8222/                        — Rust plane
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

Five named volumes hold all state — see `volumes.yaml`: `lawsynth_api_data`
(API SQLite + object store), plus `lawsynth_pgdata`, `lawsynth_minio`,
`lawsynth_nats`, `lawsynth_artifacts` (Rust plane). Remove them with
`docker compose down --volumes` to get a clean slate.

## Notes and boundaries

- **The API is SQLite + filesystem, not Postgres + S3.** It reads
  `LAWSYNTH_DATABASE_URL` (SQLite only) and `LAWSYNTH_OBJECT_ROOT`; it ignores
  the `LAWSYNTH_S3_*`, `LAWSYNTH_EVENT_BUS_URL`, and `LAWSYNTH_ARTIFACT_ENDPOINT`
  variables. Discovery runs on the compiled `lawsynth` engine in-process.
- **The Rust plane is not wired to the API yet.** `scheduler`, `worker`,
  `artifact`, `postgres`, `minio`, and `nats` form a distributed execution plane
  that the API's discovery path does not use; the API returns
  `501 worker_transport_unavailable` for `/v1/worker/*`. They are here so the
  full topology can be developed against, not because the API dispatches to them.
- **The API image must bundle the compiled engine.** The stock
  `deploy/docker/images/api.Dockerfile` builds `lawsynth._native` from the Rust
  workspace (a several-minute first build). Without it the API cannot import and
  `POST /v1/runs` returns `503 native_unavailable`.
- Studio is not started here; run it from the pnpm workspace (`apps/studio`)
  and point it at `http://localhost:8081`, or add the `studio` image yourself.
- The gateway image bundles the API package and wraps it in the admission
  layer; that is why the gateway also receives database/object-store settings.
- TLS, real secret management, backups, and rate-limit tuning belong to the
  production overlay, not this stack.
