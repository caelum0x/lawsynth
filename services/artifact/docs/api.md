# API contract

Two equivalent surfaces exist over one core: the `ArtifactService` Rust API and
the HTTP/1.1 routes served by `ArtifactServer`. Both are deterministic in the
caller-supplied Unix time. HTTP routes are defined in `src/router.rs`.

## HTTP routes

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/health` | catalog count, stored bytes, and capacity ceiling |
| POST | `/artifacts` | ingest a body; returns `201` with metadata and `Location` |
| GET | `/artifacts/{id}` | download bytes; sets `ETag` and content type |
| DELETE | `/artifacts/{id}` | remove an artifact; `204` or `404` |
| GET | `/artifacts/{id}/metadata` | metadata JSON for one artifact |
| POST | `/uploads` | begin a multipart session; returns an `upload_id` |
| PUT | `/uploads/{id}/parts/{n}` | upload part `n`; `204` on accept |
| POST | `/uploads/{id}/complete` | assemble parts into a stored artifact |
| DELETE | `/uploads/{id}` | abort a multipart session |
| POST | `/gc` | run a retention sweep; `?dry_run=true` reports only |
| GET | `/` | `{"service":"lawsynth-artifact"}` liveness banner |

Each route accepts exactly one method; other methods return `405` with an
`Allow` header. Unknown paths return `404`.

## Ingest options (headers)

`POST /artifacts` and `POST /uploads` read two optional headers:

- `Content-Type` — stored verbatim and returned on download; absent means
  `application/octet-stream`.
- `X-Retention-Expires-At` — an unsigned Unix timestamp after which the artifact
  is eligible for GC. A non-numeric value is rejected with
  `400 invalid_metadata`.

## Metadata shape

```json
{
  "id": "<sha256-hex>",
  "sha256": "<sha256-hex>",
  "size_bytes": 1234,
  "created_at_unix_seconds": 1723900000,
  "content_type": "application/json",
  "expires_at_unix_seconds": null
}
```

## Rust API

Construct `ArtifactService::open(ArtifactConfig::new(root))`, then call `ingest`,
`get`, `describe`, `delete`, the `begin_multipart` / `add_multipart_part` /
`complete_multipart` / `abort_multipart` flow, `collect_garbage`, and `health`.
The service takes `now` explicitly wherever expiry matters. See `docs/failures.md`
for the error-to-status mapping.
