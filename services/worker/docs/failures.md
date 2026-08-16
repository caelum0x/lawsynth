# Failure semantics

Invalid identifiers, expired deadlines, cancelled requests, and capacity exhaustion
do not execute work. They persist `Cancelled` or `Rejected` lifecycle records when a
valid job ID is available. Engine errors persist `Failed`; successful operations
persist `Completed` only after the engine returns a typed result.

Corrupt checkpoint bytes are rejected on read. A previous record, including an
interrupted `Running` record, prevents accidental duplicate execution. Recovery or
retry policy belongs to a higher-level workflow with a full durable input/result
codec.
