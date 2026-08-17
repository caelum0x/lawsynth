# API contract

The gateway does not define an application API of its own. It admits, canonical-
izes, rate-limits, and relays requests to the in-process `lawsynth-api` WSGI
backend, and adds only two operational endpoints. Everything else must match the
API's `/v1` surface (see `services/api/docs/api.md`).

## Routable paths

| Method | Path | Handled by |
| --- | --- | --- |
| GET | `/healthz` | gateway — liveness, always `200` while the process runs |
| GET | `/readyz` | gateway — `200` when accepting, `503` while draining |
| any | `/v1` and `/v1/*` | relayed to the API backend after admission |
| OPTIONS | `/v1/*` | gateway — CORS preflight |

Any other path returns `404 route_not_found`: the gateway exposes only the
configured `api_prefix` (default `/v1`) plus the two operational endpoints.

## Health and readiness

- `GET /healthz` -> `{"status": "ok", "accepting": <bool>, "request_id": ...}`.
- `GET /readyz` -> `{"status": "ready"|"draining", "request_id": ...}` with a
  `200`/`503` status. After `close()` is called the gateway stops accepting `/v1`
  traffic (`503 gateway_draining`) while `/healthz` and `/readyz` still answer.

## CORS

CORS is exact-origin. A request whose `Origin` is not in `allowed_origins` is
rejected `403 origin_forbidden`. A preflight (`OPTIONS`) requires an allowed
`Origin` and a valid `Access-Control-Request-Method`; it returns `204` with
`Access-Control-Allow-Methods: GET, POST, PATCH, DELETE`, the echoed requested
headers, and `Access-Control-Max-Age: 600`. On allowed origins the relayed
response gains `Access-Control-Allow-Origin` and `Vary: Origin`.

## Relay contract

For `/v1` traffic the gateway builds a fresh WSGI `environ` (dropping hop-by-hop
and client-supplied forwarding headers), injects `X-Request-Id`,
`X-Forwarded-For` (the admitted client), and `X-Forwarded-Proto: http`, invokes
the backend, then relays the backend's status, cleaned headers, and body. The
backend's `Content-Length` and `Server` headers are recomputed/stripped; the
gateway's own `X-Request-Id` is authoritative and cannot be overridden.

## Error envelope

Gateway-generated errors use the same shape as the API:
`{"error": {"code", "message", "request_id"}}` with `X-Request-Id`. See
`docs/failures.md` for the code list.
