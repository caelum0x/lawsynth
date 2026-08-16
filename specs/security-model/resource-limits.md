# Resource limits

`lawsynth_core::ResourceLimits` validates nonzero maxima for samples, columns,
features, and candidates and offers explicit validation methods. They are part
of `EngineConfig`; callers must pass and apply them to each applicable stage.
They are not an operating-system memory or CPU quota.

`lawsynth_runner::ResourceRequest` requires positive CPU and memory values.
`ResourceLimiter` admits a request only while capacity permits it, and
`execute` reserves before processing then releases afterward. Runner requests
are accounting values, not cgroups, rlimits, disk quotas, preemption, or a
wall-clock deadline enforcer. Use OS/container controls for hostile workloads.
