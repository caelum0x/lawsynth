# LawSynth

Discover governing laws from data. Run them as executable worlds.

## Current engine slice

The initial Rust workspace implements the correctness-sensitive execution path:

- portable, validated World IR identifiers;
- deterministic scalar expression evaluation;
- continuous state variables, parameters, and transition laws;
- fourth-order Runge-Kutta simulation;
- constant scenario inputs and parameter interventions.

The first public surface is intentionally small while the World IR and bundle
specifications are refined. Discovery, Python bindings, persistence, and Studio
are the next layers.

## Verify

Run `cargo test --workspace` from the repository root.

## License

Apache-2.0.
# lawsynth
