# API contract

Two surfaces exist over one core: the in-process `Scheduler<S>` Rust API and a
dependency-free HTTP/1.1 **control plane**. Routing is a pure function of
`(scheduler, now_ms, request)` (`src/router.rs`), so the whole surface is testable
without a socket. Time is supplied by the caller (Unix milliseconds), keeping the
scheduler deterministic.

## Control-plane boundary (important)

The HTTP transport exposes ONLY the scheduler's serializable operations. Lease
acquisition, heartbeat, complete, and fail are intentionally absent from the
route table: they carry or fence executable `JobEnvelope` values, which have no
wire codec. Dispatch of executable work therefore stays an in-process Rust API
call; the network surface is limited to state a client can safely observe and
mutate.

## HTTP routes

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/` | `{"service":"lawsynth-scheduler"}` liveness banner |
| GET | `/health` | `queued_count` and the effective `config` |
| POST | `/pools` | register a worker pool (metadata only; never executable work) |
| GET | `/jobs/{id}` | a job's current lifecycle state |
| GET | `/jobs/{id}/checkpoint` | a job's durable checkpoint (no payload) |
| POST | `/jobs/{id}/cancel` | cancel queued/leased work from a `{reason}` body |
| POST | `/recover` | reclaim expired leases; returns `{"recovered": n}` |

Each route accepts exactly one method; others return `405 method_not_allowed`
with an `Allow` header. Unknown paths return `404 not_found`.

## Request bodies

- `POST /pools` — `{"id": <string>, "cpu_millis": <u32>, "memory_bytes": <u64>,
  "disk_bytes": <u64>}`. On success returns `201` with the pool `id` and a
  `Location: /pools/{id}` header. This is capacity metadata; it never accepts a
  job.
- `POST /jobs/{id}/cancel` — `{"reason": <string>}`. Returns the job's resulting
  state. Cancellation is a control-plane transition; interrupting an *executing*
  worker remains that worker's cooperative responsibility.

Bodies are flat JSON objects; malformed or non-UTF-8 bodies return
`400 invalid_body`.

## Job state shape

`state` is a tagged object: `{"name":"queued"}`,
`{"name":"leased","worker_id","generation","expires_at_ms"}`,
`{"name":"completed"}`, `{"name":"cancelled","reason"}`, or
`{"name":"dead_letter","reason"}`. A checkpoint adds `job_id`, `attempt`,
`sequence`, and `updated_at_ms` around that state.

## Rust API

Construct `Scheduler::new(SchedulerConfig, store)`, then use `register_pool`,
`state`, `checkpoint`, `cancel`, `recover_expired`, and the in-process
lease/heartbeat/complete/fail methods that move typed `JobEnvelope` values. See
`docs/failures.md` for the error-to-status mapping.
