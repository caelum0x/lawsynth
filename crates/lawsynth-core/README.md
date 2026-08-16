# lawsynth-core

Deterministic primitives shared by the LawSynth engine. This crate owns stable
identifiers, seeded random streams, version values, resource limits, progress
events, diagnostics, and cancellation. It intentionally has no knowledge of
datasets, equations, or file formats.

## Use

```rust
use lawsynth_core::{Identifier, ResourceLimits, Seed, stable_hash};

let state = Identifier::new("population")?;
let seed = Seed::new(42);
let limits = ResourceLimits::default();
assert_ne!(stable_hash(&state.to_string()), 0);
# Ok::<(), lawsynth_core::IdentifierError>(())
```

`Identifier::new` validates portable names up front. `Seed` makes randomized
searches reproducible; derive child streams rather than sharing mutable global
RNG state. Apply `ResourceLimits` at public boundaries and report recoverable
conditions through `Diagnostics` or a typed error.

This crate is the dependency floor of the workspace: its public values are
serializable-by-convention primitives, not an application configuration layer.
