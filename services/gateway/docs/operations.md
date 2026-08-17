# Operations

## Running

The gateway is composed in the same process as the API and served by one WSGI
server. For local development, a loopback-only entry point wires the gateway to a
backend import target:

```sh
lawsynth-gateway --host 127.0.0.1 --port 8081 \
  --backend-module lawsynth_api.main:application
```

It refuses any non-loopback host on purpose. For a deployment, expose the
composed application from a module and run a production WSGI server behind TLS:

```python
# wsgi.py
from lawsynth_api import create_wsgi_app
from lawsynth_gateway import create_gateway
application = create_gateway(create_wsgi_app())
```

```sh
gunicorn --bind 0.0.0.0:8080 --workers 1 wsgi:application
```

There is no remote-proxy mode. If TLS termination, upstream retries, or a
separate network hop to the API are required, put an audited edge proxy in front;
the gateway will not simulate them.

## Configuration

All settings come from environment variables at startup
(`GatewaySettings.from_environment`, `settings.py`); the process reads no YAML.
`config/` documents the per-environment shape and the variable each field maps
to: `LAWSYNTH_GATEWAY_MAX_BODY_BYTES`, `_MAX_HEADER_BYTES`, `_MAX_HEADERS`,
`_MAX_CLIENTS`, `_REQUESTS_PER_WINDOW`, `_RATE_WINDOW_SECONDS`,
`_ALLOWED_ORIGINS`. `api_prefix` is fixed at `/v1`.

## Health, readiness, and draining

Point the liveness probe at `GET /healthz` and the readiness probe at
`GET /readyz`. To drain gracefully, call `GatewayApplication.close()` (the dev
entry point does this on shutdown): the gateway stops accepting `/v1` traffic
with `503 gateway_draining`, `/readyz` flips to `503 draining`, and the wrapped
API backend is closed once.

## Rate limiting

The sliding window is per client address, bounded to `max_clients` buckets by an
LRU. Tune `requests_per_window` and `rate_window_seconds` to the front door's
capacity; a rejected request returns `429` with `Retry-After`. Because the
key-space is bounded, a flood of distinct client addresses evicts old buckets
rather than growing memory without limit.

## CORS

Set `allowed_origins` to the exact browser origins (scheme + host[:port], no
path). Non-browser callers do not send `Origin` and are unaffected. An empty
allowlist blocks all cross-origin browser access.

## Observability

Correlate on `X-Request-Id`; the gateway owns it and forwards `X-Forwarded-For`
and `X-Forwarded-Proto` to the backend. Access/error logging is owned by the
hosting WSGI server — see `config/logging.yaml`.
