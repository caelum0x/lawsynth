# Failure semantics

Every domain failure is a `SchedulerError` (`src/error.rs`) translated to a
stable HTTP status and a machine-readable body by `src/http_error.rs`. The body
carries a `code` and a `message` (the error's `Display`); job payloads and
checkpoint bytes are never included.

## Error to status mapping

| `SchedulerError` | Status | code | Meaning |
| --- | --- | --- | --- |
| `InvalidConfig` | 400 | `invalid_config` | configuration failed validation |
| `InvalidWorker` | 400 | `invalid_worker` | malformed worker pool registration |
| `UnknownJob` | 404 | `unknown_job` | no job with that id |
| `UnknownWorker` | 404 | `unknown_worker` | no such registered pool |
| `DuplicateJob` | 409 | `duplicate_job` | job id already known |
| `QueueFull` | 409 | `queue_full` | `maximum_queued_jobs` reached |
| `StaleLease` | 409 | `stale_lease` | lease fenced by a newer generation |
| `LeaseExpired` | 409 | `lease_expired` | lease past its expiry |
| `InvalidTransition` | 409 | `invalid_transition` | illegal lifecycle move |
| `CheckpointTooLarge` | 413 | `checkpoint_too_large` | record exceeds `maximum_checkpoint_bytes` |
| `CorruptCheckpoint` | 500 | `corrupt_checkpoint` | a durable record failed to parse |
| `Store` | 500 | `store_error` | underlying `lawsynth-store` failure |
| `UnsupportedTransport` | 501 | `unsupported_transport` | a transport that is not linked |

Transport-level problems discovered before a domain call are answered directly:
an unmatched route is `404 not_found`, a wrong method is `405 method_not_allowed`
(with `Allow`), a malformed/oversized body is `400 invalid_body` or
`413 payload_too_large`.

## Determinism and recovery

Expiry recovery is deterministic in the supplied `now_ms`: `POST /recover`
re-queues every lease whose `expires_at_ms` has passed and reports how many were
recovered. A job is dead-lettered only after `maximum_attempts` is exhausted.
Because cancellation is a control-plane transition, cancelling a `leased` job
records the transition but does not forcibly stop an executing worker — the
worker observes cancellation cooperatively (see the worker service).

## Not a retry engine

The scheduler owns lease fencing and expiry, not a durable input/result codec.
Higher-level retry or workflow orchestration that needs to re-run a job from its
inputs belongs to a layer that carries a full envelope codec, not to this
control plane.
