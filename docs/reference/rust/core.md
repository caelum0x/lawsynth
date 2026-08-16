# `lawsynth-core`

The core crate owns stable identifiers, deterministic hashing and seed derivation, cancellation, resource limits, progress reporting, diagnostics, configuration validation, and version metadata. APIs validate inputs at construction so downstream crates can depend on explicit finite and identifier contracts.

Core is infrastructure, not a global runtime: it does not provide storage, networking, distributed execution, telemetry upload, or implicit configuration discovery.
