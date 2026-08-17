# Architecture

`lawsynth-gateway` is a transport-independent WSGI admission layer that wraps the
local `lawsynth-api` application in the same process. It has no business logic and
no persistence; it decides what may reach the backend and how the response is
returned to the caller.

## Composition

The gateway is composed around the API application, not deployed as a separate
network hop:

```python
from lawsynth_api import create_wsgi_app
from lawsynth_gateway import create_gateway

application = create_gateway(create_wsgi_app())
```

`GatewayApplication` wraps the backend in an `InProcessWsgiBackend`. Remote
upstream proxying, TLS termination, and retries are deliberately absent:
`InProcessWsgiBackend.remote(...)` raises `RemoteUpstreamUnavailable` rather than
pretending an unimplemented remote path succeeded. Those capabilities belong to a
separately audited edge proxy.

## Admission pipeline

Each request passes through, in order (`app.py`):

1. **Request id** — validate an inbound `X-Request-Id` or mint a UUID.
2. **`_admit`** — method allowlist, absolute/normalized path check, `api_prefix`
   routing, draining check, query validation, client-address extraction.
3. **Header canonicalization** — capitalize names, reject invalid names/values
   and duplicates, strip hop-by-hop and client forwarding headers, enforce the
   header count and byte ceilings.
4. **Body admission** — reject `Transfer-Encoding`/chunked, validate
   `Content-Length` against `max_body_bytes`, require `Content-Type` when a body
   is present.
5. **Origin check** and, for `OPTIONS`, **preflight**.
6. **Rate limit** — `BoundedRateLimiter`, a lock-protected sliding window with an
   LRU-bounded per-client key-space.
7. **Relay** — build a clean backend `environ`, invoke the backend, validate and
   clean the backend's WSGI response, relay it with safe headers.

## Determinism and safety

The rate limiter takes an injectable monotonic clock, so window behavior is
testable without sleeping. Every response gets fixed safe headers
(`Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, an authoritative
`X-Request-Id`, a computed `Content-Length`). A backend that misbehaves
(double `start_response`, non-bytes body, invalid status/headers) is turned into
a `502 backend_failure` rather than propagated.

## Trust boundary

The gateway is the public front door; the API behind it is not internet-exposed.
See `docs/security.md`.
