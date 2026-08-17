# Operations

## Running

For local development, run the loopback-only stdlib server:

```sh
lawsynth-api --host 127.0.0.1 --port 8080
```

It refuses any non-loopback host on purpose — the stdlib server is not a
production server. For a deployment, run a real WSGI server behind TLS and import
the module-level application:

```sh
gunicorn --bind 127.0.0.1:8080 --workers 1 lawsynth_api.main:application
```

Run exactly one worker. The alpha domain repositories are process-local (one
in-process SQLite catalog per process), so scale horizontally with multiple
replicas behind a load balancer, never with multiple gunicorn workers that would
each own a separate catalog.

## Configuration

All configuration is read from environment variables at startup by
`ApiSettings.from_environment` (`settings.py`); the process reads no YAML. The
files under `config/` document the intended per-environment shape and the
variable each field maps to. Key variables: `LAWSYNTH_API_ENV`,
`LAWSYNTH_DATABASE_URL`, `LAWSYNTH_OBJECT_ROOT`, `LAWSYNTH_MAX_PAGE_SIZE`,
`LAWSYNTH_MAX_UPLOAD_BYTES`, `LAWSYNTH_API_MAX_REQUEST_BYTES`,
`LAWSYNTH_API_EVENT_RETENTION`, `LAWSYNTH_API_TOKENS_JSON`, `LAWSYNTH_TELEMETRY`.
In `production` the process refuses to start unless `LAWSYNTH_DATABASE_URL` names
durable storage and `LAWSYNTH_OBJECT_ROOT` is absolute.

## Health and readiness

`GET /v1/health` reports `status` (`ok`/`degraded`) with `database` and
`storage` sub-statuses by probing the SQLite connection and ensuring the object
root is writable. Use it as the liveness/readiness probe. `/health` and
`/version` are the only unauthenticated routes.

## Observability

Every response and error carries `X-Request-ID` for correlation. Access/error
logs are owned by the hosting WSGI server (see the Dockerfile's gunicorn flags);
`config/logging.yaml` documents routing and the structural redaction rules.
In-process domain counters are enabled with `LAWSYNTH_TELEMETRY=true` and hold
only `<operation>:<status>` counts — never payload content.

## Storage and backup

The SQLite database at `LAWSYNTH_DATABASE_URL` and the object tree under
`LAWSYNTH_OBJECT_ROOT` are the durable state. Both are owned by the non-root
service user (uid 65532 in the image). Snapshot both together while the process
is quiescent; restoring both restores the full catalog and its artifacts.

## Secrets

Bearer tokens live in `LAWSYNTH_API_TOKENS_JSON`. Inject them from a secret
manager; never commit them to `config/` or `.env`. Rotate by replacing the JSON
grant map and restarting.
