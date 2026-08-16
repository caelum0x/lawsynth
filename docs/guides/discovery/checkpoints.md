# Checkpoints and recovery

The local CLI invocation is synchronous and writes its resulting `.lsworld` bundle only after discovery succeeds. It does not currently expose resumable optimizer checkpoints, a job queue, or server-side run recovery.

For long parameter sweeps, let an external workflow runner create one isolated output directory per immutable input/configuration pair. Write to a temporary filename, validate with `lawsynth inspect`, then atomically promote the bundle in the runner. Preserve stderr and the exact command for failed runs as well as successful ones.

Never resume by overwriting an existing accepted bundle. A distinct output path and content hash make it possible to distinguish a rerun from a changed configuration.
