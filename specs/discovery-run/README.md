# Discovery-run contract

The Phase 2 discovery entry point fits continuous equation worlds from a valid
`lawsynth_data::Dataset`. It profiles the input, optionally preprocesses it,
differentiates each requested state, builds a deterministic feature library,
fits sparse laws, optionally evaluates one bounded symbolic branch, and keeps
the non-dominated candidates.

The public Rust entry points are `discover`, `discover_cancellable`, and
`discover_cancellable_with_checkpoint`. The shipped CLI exposes the first of
these through `lawsynth discover`; checkpointing and cancellation are library
APIs rather than CLI flags.

This contract describes implemented behavior only. It does not promise causal
identification, stochastic laws, latent-state inference, distributed execution,
or arbitrary data-format ingestion.
