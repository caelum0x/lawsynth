# Resource limits

`DiscoveryConfig::resource_limits` is enforced before expensive profile or
feature work and again after feature/candidate construction. All four limits
must be nonzero:

- `max_samples` (default 1,000,000)
- `max_columns` (default 1,024)
- `max_features` (default 50,000)
- `max_candidates` (default 10,000)

The run rejects a dataset or generated feature/candidate count above its
limit. These bounds are item-count guardrails, not CPU-time, memory-byte,
wall-clock, recursion-depth, or cancellation deadlines. Cancellation is
cooperative and checked at stage boundaries and during per-state/bootstrap
work; it is not preemptive.
