# Failure semantics

Every domain failure is an `ArtifactError` (`src/errors.rs`) translated to a
stable HTTP status and a machine-readable body through `src/http_error.rs`. The
body carries a `code` and `message`; object bytes and secrets are never included.

## Error to status mapping

| `ArtifactError` | Typical status | Meaning |
| --- | --- | --- |
| `InvalidConfig` | startup abort | configuration failed `validate` before serving |
| `InvalidArtifactId` | `400` | malformed identifier in the path |
| `InvalidMetadata` | `400` | bad retention header or unsupported principal |
| `InvalidUpload` | `400` | malformed or out-of-order multipart request |
| `NotFound` | `404` | no artifact or upload session with that id |
| `Expired` | `404`/`410` | artifact past its retention timestamp |
| `ChecksumMismatch` | `500` | stored bytes did not match the recorded digest |
| `CapacityExceeded` | `413` | write would exceed the configured ceiling |
| `CorruptMetadata` | `500` | a durable metadata record failed to parse |
| `Store` | mapped by kind | underlying `lawsynth-store` failure |

Transport-level problems discovered before a domain call — an unmatched route,
an unsupported method, a non-integer part number — are answered directly with
`404`, `405 method_not_allowed` (with an `Allow` header), or `400 invalid_part`.

## Integrity and idempotency

Content addressing makes ingest naturally idempotent: identical bytes produce the
same id. A checksum mismatch on read is surfaced as an error and counted in
`telemetry.checksum_failures` rather than returning corrupt data. A GC sweep is
deterministic in the supplied `now`; `--dry-run` reports the eligible ids without
deleting them so an operator can review a sweep before committing to it.
