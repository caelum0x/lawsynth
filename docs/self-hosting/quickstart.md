# Quickstart: get LawSynth running and make your first discovery over HTTP

This walks you from nothing to a **discovered mathematical world served over
HTTP** in a few minutes: start the stack, mint a token, submit a discovery run
with `curl`, poll it to completion, then fetch the world and its self-contained
HTML report. Every request here has been run against the real services.

## What you're bringing up

```
client ──▶ gateway :8081 (admission) ──▶ api :8080 (discovery-as-a-service)
                                          ├─ SQLite  metadata     (/data/metadata.sqlite3)
                                          └─ files   object store  (/data/objects)
```

The LawSynth **API** is a Python WSGI process over the `lawsynth-server` domain
core. It keeps metadata in **SQLite**, stores content-addressed objects on the
**filesystem**, and runs discovery on the compiled `lawsynth` engine
**in-process**. The **gateway** is a thin admission layer (body/header limits,
a per-client rate window, CORS) in front of it.

> **Honest scope.** This is not the full distributed graph. The API does **not**
> use Postgres, S3/MinIO, or NATS — those back the separate Rust
> scheduler/worker/artifact plane, which is not yet wired to this API's discovery
> path. The API returns `501 worker_transport_unavailable` for `/v1/worker/*`
> on purpose. For the aspirational full stack see
> [`deploy/compose/local`](../../deploy/compose/local/).

---

## 1. Start the stack

You need the compiled `lawsynth` engine available to the API. Pick one path.

### Option A — Docker Compose (self-contained)

The quickstart image compiles the engine for you. From the repository root:

```bash
cd deploy/quickstart
cp .env.example .env
# Edit .env and set LAWSYNTH_API_TOKEN to a long random value, e.g.:
#   LAWSYNTH_API_TOKEN=$(openssl rand -hex 24)
docker compose -f compose.yaml up --build -d
```

> The **first build compiles the numeric Rust crates** (several minutes). It
> bakes the PyO3 extension into the image so `POST /v1/runs` can run real
> discovery instead of returning `503 native_unavailable`.

Validate the compose config any time without a running daemon:

```bash
docker compose -f compose.yaml config -q
```

### Option B — Run from source (fastest to verify, no container build)

Requires Python 3.11+ and a Rust toolchain. From the repository root:

```bash
# 1) Build the native engine next to the pure-Python package.
python/lawsynth/scripts/build-native.sh          # cargo build -p lawsynth-python (release)

# 2) Put the three Python packages on the path.
export PYTHONPATH="python/lawsynth/src:python/lawsynth-server/src:services/api/src"

# 3) Configure a token, durable SQLite, and an object root.
export LAWSYNTH_API_ENV=development
export LAWSYNTH_DATABASE_URL="sqlite:///$PWD/.lawsynth/metadata.sqlite3"
export LAWSYNTH_OBJECT_ROOT="$PWD/.lawsynth/objects"
export LAWSYNTH_API_TOKENS_JSON='{"quickstart-local-token-change-me":{"organization_id":"local","scopes":["read","write"]}}'
mkdir -p .lawsynth

# 4) Run the loopback development server (API on :8080).
python -m lawsynth_api.main --host 127.0.0.1 --port 8080
```

For a production-style run use gunicorn behind TLS (one worker — the domain
repositories and the in-process discovery/event bus are process-local):

```bash
gunicorn --bind 127.0.0.1:8080 --workers 1 lawsynth_api.main:application
```

To also front it with the admission gateway, add `services/gateway/src` to
`PYTHONPATH` and run:

```bash
python -m lawsynth_gateway.main --host 127.0.0.1 --port 8081 \
    --backend-module lawsynth_api.main:application
```

---

## 2. Set your base URL and token

Talk to the **gateway** (`:8081`) the way a client would; the direct API
(`:8080`) is identical minus admission. Option B without the gateway: use
`:8080`.

```bash
export BASE=http://localhost:8081
export TOKEN=quickstart-local-token-change-me   # must match your .env / TOKENS_JSON
```

A token is a **tenant grant**: it maps to one `organization_id` and a set of
scopes (`read`, `write`, `admin`). Configure grants through
`LAWSYNTH_API_TOKENS_JSON` (tokens must be at least 16 characters):

```json
{"a-long-secret-token": {"organization_id": "acme", "scopes": ["read", "write"]}}
```

Writes (`POST`/`PATCH`/`DELETE`) require **both** the bearer token and an
`Idempotency-Key` header.

---

## 3. Check health and version (no auth needed)

```bash
curl -s $BASE/healthz          # gateway's own liveness -> {"status":"ok","accepting":true,...}
curl -s $BASE/v1/health        # {"status":"ok","database":"ok","storage":"ok"}
curl -s $BASE/v1/version       # {"version":"0.1.0","protocol":"1"}
```

---

## 4. Submit a discovery run — `POST /v1/runs`

A discovery run names the **state** columns to model and points at a dataset —
either an inline dataset (a `csv` string, or `time` + `columns`) or an already
uploaded `dataset_id`. Below, an inline dataset of simple exponential decay
(`x(t) = e^(-0.5 t)`, so the true law is `dx/dt = -0.5·x`):

```bash
# Build the request body (60 samples of x = exp(-0.5 t)).
python3 - > body.json <<'PY'
import json, math
t = [round(0.1 * i, 4) for i in range(60)]
x = [round(math.exp(-0.5 * ti), 6) for ti in t]
print(json.dumps({
    "name": "decay-demo",
    "states": ["x"],
    "dataset": {"name": "decay", "time": t, "columns": {"x": x}},
    "discovery": {"polynomial_degree": 2, "threshold": 0.05}
}))
PY

curl -s -X POST $BASE/v1/runs \
  -H "Authorization: Bearer $TOKEN" \
  -H "Idempotency-Key: decay-demo-001" \
  -H "Content-Type: application/json" \
  --data @body.json
```

The run is created in `queued`; a background thread runs discovery and moves it
`queued → running → succeeded|failed`. The response is the run record:

```json
{
  "id": "5759dde7-4d5b-45e2-acb6-004c0f594c2d",
  "name": "decay-demo",
  "status": "queued",
  "dataset_id": "205bec63-52ef-4332-bf0f-46407bbcc58d",
  "metadata": {"kind": "discovery", "phase": "queued", "states": ["x"], "config": {...}},
  "organization_id": "local"
}
```

Capture the id:

```bash
RID=$(curl -s -X POST $BASE/v1/runs \
  -H "Authorization: Bearer $TOKEN" -H "Idempotency-Key: decay-demo-002" \
  -H "Content-Type: application/json" --data @body.json \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['id'])")
```

**Request fields**

| Field | Required | Notes |
|---|---|---|
| `states` | yes | Non-empty list of identifiers; each must be a dataset column. |
| `dataset` **or** `dataset_id` | yes (exactly one) | Inline (`csv`, or `time`+`columns`) or a prior upload. |
| `discovery` | no | Engine knobs: `polynomial_degree`, `threshold`, `solver`, `include_trigonometric`, `include_rational`, `derivative_method`, … Also accepts `recipe`/`preset` and the alias `degree`. |
| `name`, `world_name`, `project_id` | no | Optional labels; `project_id` must reference an existing project. |

---

## 5. Poll the run — `GET /v1/runs/{id}`

```bash
curl -s $BASE/v1/runs/$RID -H "Authorization: Bearer $TOKEN"
```

When `status` is `succeeded`, `world_id` is populated and `metadata.summary`
carries the result — laws, complexity, in-sample `mse`, and the recovered
equations:

```json
{
  "status": "succeeded",
  "world_id": "3d521ae6-fdef-460b-8b34-254bbedbd8aa",
  "metadata": {
    "phase": "succeeded",
    "summary": {
      "laws": 1,
      "complexity": {"laws": 1, "total_terms": 1},
      "mse": 1.35e-08,
      "equations": {"x": "(-5.0020811e-1*x)"}
    }
  }
}
```

`dx/dt = -0.5002·x` — the true law recovered to ~4 significant figures.

---

## 6. Fetch the world, explain it, and get the report

```bash
# The world a run discovered, with product links.
curl -s $BASE/v1/runs/$RID/world -H "Authorization: Bearer $TOKEN"

WID=$(curl -s $BASE/v1/runs/$RID/world -H "Authorization: Bearer $TOKEN" \
  | python3 -c "import sys,json; print(json.load(sys.stdin)['world_id'])")

# Plain-language explanation (readable laws, dependencies, complexity).
curl -s $BASE/v1/worlds/$WID/explain -H "Authorization: Bearer $TOKEN"

# Self-contained HTML report (rendered equations + inline SVG, no external assets).
curl -s $BASE/v1/worlds/$WID/report -H "Authorization: Bearer $TOKEN" -o decay-report.html
open decay-report.html   # or: xdg-open decay-report.html
```

`GET /v1/runs/{id}/world` returns:

```json
{
  "run_id": "5759dde7-...",
  "world_id": "3d521ae6-...",
  "world": {"states": ["x"], "equations": {"x": "(-5.0020811e-1*x)"}, ...},
  "links": {
    "self": "/v1/worlds/3d521ae6-...",
    "explain": "/v1/worlds/3d521ae6-.../explain",
    "report": "/v1/worlds/3d521ae6-.../report"
  }
}
```

`explain` reports, for this world:

```json
{"variables": ["x"], "laws": [{"readable": "dx/dt = -0.5002·x", ...}], ...}
```

Other world product routes: `POST /v1/worlds/{id}/forecast` (native simulate —
needs `write` scope), `POST /v1/worlds/compare`, and `POST /v1/worlds/{id}/simulate`.

---

## The Python client equivalent

The `lawsynth` SDK ships a dependency-free `Client` that drives the exact same
`/v1` contract (bearer auth, `X-Api-Version: 1`, the shared error envelope).
It auto-generates an `Idempotency-Key` for writes.

```python
import lawsynth

client = lawsynth.Client("http://localhost:8081", token="quickstart-local-token-change-me")

print(client.health())      # {"status": "ok", "database": "ok", "storage": "ok"}
print(client.version())     # {"version": "0.1.0", "protocol": "1"}

# Submit a discovery run from an inline dataset, then wait for it.
import math
t = [round(0.1 * i, 4) for i in range(60)]
run = client.submit_discovery(
    state=["x"],
    columns={"x": [math.exp(-0.5 * ti) for ti in t]},
    time=t,
    degree=2,
    threshold=0.05,
    name="decay-demo",
)
run = client.wait(run)                 # polls GET /v1/runs/{id} to a terminal status
assert run.succeeded, run.status
print(run.summary)                     # mse, complexity, laws, world_id

world = client.world(run)              # GET /v1/runs/{id}/world (flattened)
print(client.explain(world_id=run.world_id)["laws"])
client.report(run.world_id, "decay-report.html")   # writes the HTML report
```

For notebooks and offline tests you can drive the WSGI app in-process without a
socket: `lawsynth.Client(wsgi_app=lawsynth_api.main.create_wsgi_app())`.

---

## Deterministic, offline, and air-gapped

- **Deterministic & offline.** Discovery is deterministic and runs entirely on
  the local machine — identical inputs reproduce the same world. Nothing in this
  loop reaches the network.
- **Air-gap.** To move the whole stack (images, datasets, checksums) into a
  disconnected environment, use the export/import bundle under
  [`deploy/airgap/bundle`](../../deploy/airgap/bundle/) — see
  [airgap.md](airgap.md). The bundle's `export.sh`, `import.sh`, and `verify.sh`
  package the container images and verify checksums on the far side.

---

## Troubleshooting

| Symptom | Cause & fix |
|---|---|
| `503 native_unavailable` on `POST /v1/runs` | The compiled `lawsynth` engine isn't importable. Use the quickstart image (Option A) or run `python/lawsynth/scripts/build-native.sh` (Option B). The stock `deploy/docker/images/api.Dockerfile` does **not** include the engine. |
| API container/process won't start (`ImportError: lawsynth`) | Same cause — the API imports `lawsynth.report` at load. Build the engine as above. |
| `401` unauthenticated | Missing/incorrect `Authorization: Bearer <token>`; the token must appear in `LAWSYNTH_API_TOKENS_JSON` and be ≥ 16 chars. |
| `422 Idempotency-Key is required for writes` | Add an `Idempotency-Key` header to every `POST`/`PATCH`/`DELETE`. Reusing a key safely replays the same result. |
| `403 origin_forbidden` from the gateway | Your browser `Origin` isn't in `LAWSYNTH_GATEWAY_ALLOWED_ORIGINS`. Plain `curl` (no `Origin`) is unaffected. |
| `409` on `GET /v1/runs/{id}/world` | The run hasn't produced a world yet (still `queued`/`running`) or it `failed`. Poll `GET /v1/runs/{id}` first. |

## Where to next

- [architecture.md](architecture.md) — how the server core is structured.
- [authentication.md](authentication.md) — token grants and scopes.
- [storage.md](storage.md) / [database.md](database.md) — the object root and SQLite metadata.
- [`deploy/compose/local`](../../deploy/compose/local/) — the fuller multi-service stack.
