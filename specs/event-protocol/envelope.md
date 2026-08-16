# Work envelopes

`lawsynth_runner::WorkEnvelope` is the implemented admission record for one
in-process work item. Its fields are `id`, `kind`, `attempt`,
`submitted_at_ms`, `deadline_at_ms`, `resources`, and opaque `input` bytes.
Construction rejects empty or non URL-safe identifiers, identifiers longer
than 128 bytes, attempt zero, a non-increasing deadline, and input larger than
64 MiB. It does not authenticate the producer, parse the input, or enforce a
clock source.

`is_expired(now_ms)` is a comparison helper: expiration is true when
`now_ms >= deadline_at_ms`. `execute` does not call it. A scheduler must check
deadlines before dispatch and decide how an expired item is reported.

This is an in-memory Rust type, not a serialized protocol message. There is no
canonical byte encoding, schema version field, message signing, or remote
receiver in the current implementation.
