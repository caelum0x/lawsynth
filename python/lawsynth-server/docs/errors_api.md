# Error API

Responses emitted by middleware have an `error` object with a stable `code`, a
human-readable `message`, optional safe `details`, and an `X-Request-ID`.
Authentication failures are 401, authorization failures 403, validation errors
422, missing resources 404, and idempotency/name conflicts 409.
