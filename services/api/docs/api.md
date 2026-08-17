# API contract

`lawsynth-api` is a stdlib WSGI adapter over `lawsynth_server.Application`. It
adds no routes of its own except the streaming endpoint and an explicit marker
for the not-yet-deployed worker transport; every other operation is dispatched
unchanged to the domain core (`app.py`). All routes are served under the `/v1`
prefix (unprefixed aliases exist for `/health` and `/version`).

## Versioning

Every response carries `X-Api-Version: 1`. A request may send `X-Api-Version`
with `1` or `v1`; any other value is rejected `406 unsupported_api_version`.
`GET /v1/version` returns `{"version": <package>, "protocol": "1"}`.

## Routes

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/v1/health` | database + storage readiness (`ok`/`degraded`) |
| GET | `/v1/version` | package version and protocol version |
| GET | `/v1/projects` `/v1/datasets` `/v1/worlds` `/v1/runs` | list (paginated) |
| POST | `/v1/projects` `/v1/datasets` `/v1/worlds` `/v1/runs` | create |
| GET | `/v1/{collection}/{id}` | fetch one |
| PATCH | `/v1/{collection}/{id}` | update one |
| DELETE | `/v1/{collection}/{id}` | delete one (`204`) |
| POST | `/v1/worlds/{id}/simulate` | run the RK4 simulator for a world |
| POST | `/v1/runs/{id}/cancel` | cancel a non-terminal run |
| GET | `/v1/runs/{id}/events` | events filtered to one run |
| POST | `/v1/artifacts` | upload base64 bytes; returns `sha256` metadata |
| GET | `/v1/artifacts/{sha256}` | download bytes (base64 in JSON) |
| GET | `/v1/events` | list events, or SSE stream (see below) |

`/v1/worker` and `/v1/worker/*` always return `501 worker_transport_unavailable`:
this process is not a worker gateway.

## Authentication

Every route except `/health` and `/version` requires
`Authorization: Bearer <token>`. Tokens are provisioned through
`LAWSYNTH_API_TOKENS_JSON` and map to one `organization_id` (the tenant) and a
set of scopes (`read`, `write`, `admin`). Reads require `read`; mutations require
`write`; `admin` implies both. Missing/invalid tokens return `401`; insufficient
scope returns `403`.

## Writes, idempotency, media type

Request bodies must be `application/json` objects. Every mutating request must
send an `Idempotency-Key` header; the domain replays a stored result for a
repeated key and marks the response `Idempotency-Replayed: true`. Bodies larger
than `max_request_bytes` are rejected `413`.

## Pagination

List routes accept `?limit=<n>&cursor=<opaque>`. Responses are enveloped as
`{"items": [...], "next_cursor": <opaque|null>, "total": <n>, "limit": <n>}`.
`limit` must be within `1..max_page_size`; `next_cursor` is an opaque base64
offset to pass back for the next page.

## Events (SSE)

`GET /v1/events` with `Accept: text/event-stream` returns, as framed SSE, every
retained event for the caller's tenant whose sequence is greater than the
`Last-Event-ID` request header (0 when absent), then closes the connection — the
socket is not held open. Clients resume by reconnecting with the last id they
saw. Retention is bounded and in-process (`event_stream_retention`); evicted
events are not replayable. Without the SSE `Accept` header the same path returns
a plain JSON `{"items": [...]}` list.

## Error envelope

Every error body is `{"error": {"code": <string>, "message": <string>,
"request_id": <uuid>}}` with the same `request_id` echoed in the `X-Request-ID`
header. See `docs/failures.md` for the code-to-status mapping.
