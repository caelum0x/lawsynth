# Events and cancellation

Discovery cancellation uses `lawsynth_core::CancellationToken`. The cancellable
entry points check it before validation, after profiling/features, before each
state fit, and inside bootstrap and symbolic loops. A cancelled run returns
`DiscoveryError::Cancelled`; already written checkpoint state remains available
to the caller-owned checkpoint object.

The core `ProgressTracker` can create `ProgressEvent { sequence, stage,
fraction, message }` values and rejects decreasing fractions within a stage.
Sequence numbers are assigned deterministically. The current discovery
executor does not accept an event callback and does not publish these events.
There is therefore no wire event schema, event persistence, or CLI streaming
output in the implemented discovery path.
