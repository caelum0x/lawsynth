# Failure semantics

The gateway answers a request either from admission (a `Problem` raised in
`app.py`) or by relaying the backend's response. Gateway-generated failures use
`{"error": {"code", "message", "request_id"}}` with `X-Request-Id`.

## Admission failures

| Status | code | Cause |
| --- | --- | --- |
| 400 | `invalid_path` | path is not absolute/normalized, or contains control chars, `\`, `.`/`..` |
| 400 | `invalid_query` | query contains control characters |
| 400 | `invalid_header` / `duplicate_header` | bad header name/value or a repeated header |
| 400 | `invalid_content_length` / `invalid_request_body` | body does not match `Content-Length` |
| 400 | `unsupported_transfer_encoding` | chunked or transformed body |
| 400 | `invalid_preflight` | OPTIONS lacked an allowed origin/method or requested a forbidden header |
| 403 | `origin_forbidden` | `Origin` is not in `allowed_origins` |
| 404 | `route_not_found` | path is outside `api_prefix`, `/healthz`, `/readyz` |
| 405 | `method_not_allowed` | method not in GET/POST/PATCH/DELETE/OPTIONS |
| 413 | `payload_too_large` | body exceeds `max_body_bytes` |
| 415 | `missing_content_type` | a body was sent without `Content-Type` |
| 429 | `rate_limited` | sliding-window limit exceeded (includes `Retry-After`) |
| 431 | `too_many_headers` / `headers_too_large` | header count or byte ceiling exceeded |
| 503 | `gateway_draining` | `close()` was called; `/v1` traffic is no longer accepted |

## Backend and internal failures

| Status | code | Cause |
| --- | --- | --- |
| 502 | `backend_failure` | the backend raised, called `start_response` twice, or returned an invalid status/headers/body |
| 500 | `internal_error` | the gateway could not serialize its own response |

`/healthz` and `/readyz` remain answerable even while draining, so an
orchestrator can observe the drain state (`readyz` -> `503 draining`) without
being told the whole process is down.

## Relayed responses

For an admitted `/v1` request the status, headers, and body come from the API
backend unchanged in meaning; the gateway only strips hop-by-hop / `Content-Length`
/ `Server` headers, recomputes `Content-Length`, and stamps its authoritative
`X-Request-Id`. See `services/api/docs/failures.md` for the backend's own codes.
