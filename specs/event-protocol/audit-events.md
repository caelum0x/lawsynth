# Audit boundary

No audit-event writer, immutable audit log, actor identity, request identity,
or retention policy is implemented by the current crates. `ExecutionReport`
and `ProgressEvent` are operational values, not audit records.

For an external audit system, record an authenticated actor and request at the
service boundary, retain the submitted `WorkEnvelope`, store exact artifact
bytes or their cryptographic digest, and distinguish engine results from
transport outcomes. Do not infer any of those fields from the current types.
