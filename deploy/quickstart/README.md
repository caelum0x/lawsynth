# LawSynth quickstart profile

The smallest Compose stack that actually serves a **discovery over HTTP**: the
API (discovery-as-a-service) behind the admission gateway, backed by SQLite and
a filesystem object store.

```
client ──▶ gateway :8081 (admission) ──▶ api :8080 (discovery-as-a-service)
                                          ├─ SQLite  metadata    (/data/metadata.sqlite3)
                                          └─ files   object store (/data/objects)
```

For the end-to-end walkthrough (start the stack, mint a token, submit a run via
`curl`, poll it, fetch the world + report), see
[`docs/self-hosting/quickstart.md`](../../docs/self-hosting/quickstart.md).

## What this is (and isn't)

- **Is:** a single image bundling the compiled `lawsynth` engine plus the
  Python `lawsynth-server` core, the WSGI API, and the admission gateway. Both
  services run from that one image and share one data volume.
- **Isn't:** the full distributed graph. The LawSynth API keeps metadata in
  **SQLite** and objects on the **filesystem**, and runs discovery **in-process**
  on the native engine. It does **not** use Postgres, S3/MinIO, or NATS — those
  back the separate Rust scheduler/worker/artifact plane, which is not wired to
  this API's discovery path. See [`../compose/local`](../compose/local/) for the
  aspirational full stack and its boundary notes.

## The build step is real

The stock `deploy/docker/images/api.Dockerfile` installs only the pure-Python
packages, so its image cannot import the API (`lawsynth.report` is imported at
module load) and every run would return `503 native_unavailable`. This profile's
[`Dockerfile`](Dockerfile) fixes that: its first stage compiles the PyO3
extension (`lawsynth._native`) from the Rust workspace with
`cargo build -p lawsynth-python --release` — the same artifact
[`python/lawsynth/scripts/build-native.sh`](../../python/lawsynth/scripts/build-native.sh)
produces — and drops it next to the pure-Python `lawsynth` package.

> The first build compiles the numeric Rust crates and takes several minutes.
> If you'd rather not build a container at all, the "run from source" path in
> the quickstart doc installs the same pieces with `pip` + `cargo` and is the
> fastest way to verify the loop.

## Files

| File | Purpose |
|---|---|
| `compose.yaml` | `api` + `gateway`, one shared image and data volume |
| `Dockerfile` | Compiles the engine, installs core + API + gateway |
| `wsgi_gateway.py` | Gunicorn shim: gateway admission wrapping the API |
| `.env.example` | Copy to `.env`; set a real `LAWSYNTH_API_TOKEN` |

## Quick start

```bash
cp .env.example .env                      # set LAWSYNTH_API_TOKEN to a long value
docker compose -f compose.yaml up --build -d
curl http://localhost:8081/healthz        # gateway liveness -> {"status":"ok",...}
curl http://localhost:8080/v1/health      # API readiness -> {"status":"ok","database":"ok","storage":"ok"}
```

Validate the config without a running daemon:

```bash
docker compose -f compose.yaml config -q
```

## Health & ports

| Endpoint | Serves | Auth |
|---|---|---|
| `GET :8081/healthz` | gateway liveness (its own, not proxied) | none |
| `GET :8081/readyz` | gateway readiness (503 while draining) | none |
| `GET :8080/v1/health` | API + database + storage readiness | none |
| `GET :8080/v1/version` | `{version, protocol}` | none |
| `GET :8081/v1/*` | proxied to the API through admission | per-route |

## Ops

```bash
docker compose -f compose.yaml ps
docker compose -f compose.yaml logs -f api
docker compose -f compose.yaml down            # stop, keep data
docker compose -f compose.yaml down --volumes  # stop AND delete data
```
