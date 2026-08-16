# Operations

Embed the worker in a process that owns its `LocalStore` root and passes an explicit
`WorkerConfig`. Size the resource capacity to the process's real cgroup or host
allocation; worker limits are admission limits, not OS enforcement.

Inspect `worker/checkpoints/<job-id>.checkpoint` through `Worker::checkpoint` rather
than editing it. The records are atomically published by the local object store and
are checked for version, shape, and escaped UTF-8 detail on read.
