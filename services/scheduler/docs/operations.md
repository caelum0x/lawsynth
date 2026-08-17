# Operations

## Running

The binary takes explicit CLI arguments and reads no environment variables:

```sh
# In-memory control plane (checkpoints live only for the process lifetime):
lawsynth-scheduler serve 0.0.0.0:8082

# Durable control plane (checkpoints persisted through a LocalStore root):
lawsynth-scheduler serve 0.0.0.0:8082 /var/lib/lawsynth/scheduler
```

`serve` binds the listener, prints one startup line to stderr, then blocks,
handling one request per connection on a thread-per-connection model. Run with no
subcommand and it prints the honest transport statement — executable dispatch is
in-process and typed; the HTTP surface is control-plane only — and exits.

The process owns its store root; run it as a dedicated non-root user that owns
that path (the Dockerfile uses uid 10001 and a `/var/lib/lawsynth/scheduler`
volume).

## Configuration

Sizing lives in `config/`. The current CLI applies the compiled-in
`SchedulerConfig::default()` and `StoreConfig::default()`; treat the profile
files as the reviewed source of those numbers. Key bounds: `maximum_queued_jobs`
(queue depth), `maximum_attempts` (before dead-letter), `lease_duration`
(expiry), and `maximum_checkpoint_bytes` (durable record ceiling, min 5 KiB).
Size `lease_duration` to exceed the worst-case in-process job runtime so
`recover` does not re-queue work that is still executing.

## Health and inspection

The control plane is the observability surface (there are no in-process
counters):

```sh
curl -s http://127.0.0.1:8082/health              # queued_count + effective config
curl -s http://127.0.0.1:8082/jobs/<id>           # a job's lifecycle state
curl -s http://127.0.0.1:8082/jobs/<id>/checkpoint  # durable checkpoint (no payload)
```

Point a readiness probe at `GET /health`.

## Recovering expired leases

A worker that dies mid-lease leaves the job `leased` until its lease expires.
Reclaim expired leases with `POST /recover`; it re-queues every lease past its
`expires_at_ms` and returns `{"recovered": n}`. Schedule it on an interval sized
below `lease_duration`.

## Backup

When run with a durable root, checkpoints are content-addressed and published
atomically by the store, so the root directory can be snapshotted with normal
filesystem tooling while the process is quiescent. Restoring the directory
restores the durable lifecycle records.
