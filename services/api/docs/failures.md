# Failure semantics

Failures fall into two groups: transport problems the WSGI adapter detects
before dispatch (`RequestProblem` in `app.py`), and domain errors raised by
`lawsynth-server` and translated by its middleware. Both produce the same
envelope: `{"error": {"code", "message", "request_id"}}` with `X-Request-ID`.

## Transport-level (adapter, before dispatch)

| Status | code | Cause |
| --- | --- | --- |
| 400 | `invalid_path` / `invalid_query` / `invalid_header` | malformed or unsafe request line, query, or header |
| 400 | `invalid_content_length` / `invalid_request_body` | body does not match `Content-Length` |
| 400 | `invalid_last_event_id` | non-numeric `Last-Event-ID` on the SSE route |
| 405 | `method_not_allowed` | method is not GET/POST/PATCH/DELETE |
| 406 | `unsupported_api_version` | `X-Api-Version` is not `1`/`v1` |
| 413 | `payload_too_large` | body exceeds `max_request_bytes` |
| 415 | `unsupported_media_type` | body is not `application/json` |
| 422 | `validation_error` | JSON body is not an object |
| 501 | `worker_transport_unavailable` | `/v1/worker/*` — no worker gateway here |
| 503 | `service_unavailable` | the process is shutting down |

## Domain errors (lawsynth-server)

| Status | code | Meaning |
| --- | --- | --- |
| 401 | `authentication_required` | missing or invalid bearer token |
| 403 | `forbidden` | token lacks the required scope |
| 404 | `not_found` | unknown route or entity |
| 409 | `conflict` | lifecycle conflict (e.g. cancelling a terminal run) |
| 409 | `idempotency_conflict` | same `Idempotency-Key` reused with a different request |
| 422 | `validation_error` | request body failed domain validation |
| 503 | `native_unavailable` | the optional executable LawSynth runtime is absent |
| 500 | `internal_error` | unexpected failure (message is generic; details are logged) |

## Idempotency

Every mutation requires an `Idempotency-Key`. A repeated key with the same
request body replays the stored result and sets `Idempotency-Replayed: true`;
the adapter suppresses re-emitting a lifecycle event for a replay. A repeated
key with a *different* request is a `409 idempotency_conflict`.

## Honest unavailability

The API returns `503 native_unavailable` rather than faking a result when the
executable native runtime is missing, and `501 worker_transport_unavailable` for
`/v1/worker/*`. Neither is a server bug — they are explicit statements that this
process does not implement that capability.
