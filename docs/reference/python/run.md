# Run records

`RunRecord.completed(identifier, kind)` produces an immutable UTC timestamped record. Valid kinds are `discovery` and `simulation`; valid statuses are `completed`, `failed`, and `cancelled`.

Run records are application metadata only. They are not emitted by the native extension and do not provide persistence, cancellation control, logs, job queues, or audit storage.
