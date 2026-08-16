# Idempotency boundary

The shared types define no idempotency key, request digest, deduplication store,
or retry protocol. Replaying a request against a future service has no implied
effect under the current local contracts.

A service with create, start, cancel, upload, or artifact-writing operations
MUST define which operations accept idempotency keys, their scope and lifetime,
and the response for a matching key with a different payload. This must be
implemented at the transactional service boundary, not inferred from RunId.
