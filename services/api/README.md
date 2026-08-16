# LawSynth API

`lawsynth-api` is the HTTP process for LawSynth's Python domain core. It is a
stdlib WSGI adapter, not a second implementation of projects, datasets,
worlds, simulations, artifacts, events, or idempotency. Those operations are
dispatched directly to `lawsynth_server.Application`.

Run the loopback-only development process after installing both local packages:

```sh
lawsynth-api --host 127.0.0.1 --port 8080
```

For a deployment, run a WSGI server behind TLS and import
`lawsynth_api.main:application`:

```sh
gunicorn --bind 127.0.0.1:8080 --workers 1 lawsynth_api.main:application
```

The alpha domain repositories are intentionally process-local, so run one API
worker behind a load balancer; the service does not claim distributed metadata
or remote worker execution it does not have.

The process accepts JSON objects only. Writes require both a bearer token and
an `Idempotency-Key`; those are passed unchanged to the domain core. Configure
local bearer grants through `LAWSYNTH_API_TOKENS_JSON`, for example:

```json
{"a-long-secret-token":{"organization_id":"acme","scopes":["read","write"]}}
```

In `production`, `LAWSYNTH_DATABASE_URL` must name durable SQLite storage and
`LAWSYNTH_OBJECT_ROOT` must be an absolute path. The API deliberately returns
`503 native_unavailable` when an executable native LawSynth runtime is absent,
and `501 worker_transport_unavailable` for `/v1/worker/*`: this process does
not pretend to be a worker gateway.
