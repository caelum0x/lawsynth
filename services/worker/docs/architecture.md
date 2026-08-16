# Architecture

The worker has three boundaries. `JobEnvelope` validates a typed discovery or
simulation request and wraps the shared runner envelope. `Worker` uses the runner's
resource limiter for admission, cooperatively observes cancellation and deadlines,
and dispatches to `lawsynth-discovery` or `lawsynth-sim`. `JobCheckpoint` stores a
versioned, strictly advancing lifecycle record in `lawsynth-store`.

Checkpoints are durable status evidence, not fake resumability. A completed world or
trajectory is returned to the in-process caller in its native typed form; no lossy
result serializer is implied. A caller that needs remote execution must provide a
separate authenticated transport and a complete codec.
