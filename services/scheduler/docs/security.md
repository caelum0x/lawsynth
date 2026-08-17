# Security boundary

The scheduler trusts only its local caller and the operators of its control
plane. It is a deliberate, narrow boundary, not a multi-tenant, authenticated
job broker.

## What the service does

- Keeps executable work in-process. The HTTP transport exposes only the
  serializable control plane (health, pool registration, job state, checkpoints,
  cancellation, expiry recovery); it never accepts or emits an executable
  `JobEnvelope`, because that value has no wire codec.
- Bounds the request line + header block and the request body at 64 KiB each in
  the transport, and bounds durable checkpoints by `maximum_checkpoint_bytes`.
- Fences leases by generation so a stale worker cannot complete or overwrite a
  job that was reassigned after its lease expired.
- Redacts by construction: job payloads and checkpoint bytes never appear in
  stderr or in error bodies, which carry only a stable `code` and message.

## What the service does not do

- No authentication or authorization. The control plane has no principals; any
  client that can reach the socket can register pools, read state, cancel jobs,
  and trigger recovery. Put an authenticated, TLS-terminating front door in front
  of it and bind to a private interface.
- No transport encryption. `serve` speaks plain HTTP/1.1; terminate TLS upstream.
- No remote executable dispatch. `SchedulerTransport::HttpControlPlane` is the
  serializable subset; `LocalTyped` (the real dispatch path) stays in-process. An
  embedder that needs remote dispatch must provide its own authenticated
  transport and a complete envelope codec — the scheduler will not fake one.
- No OS-level isolation. Pool capacities and queue bounds gate admission; they do
  not contain arbitrary code or enforce cgroup/disk quotas.

## Deployment guidance

Run as a dedicated non-root user that owns the store root, bind to a private
address behind an authenticating proxy, and never expose the control plane
directly to untrusted networks. Snapshot the durable root with the process
quiescent for backup.
