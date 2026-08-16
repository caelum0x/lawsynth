# Seed plan

`lawsynth_core::Seed` wraps a `u64`. `Seed::derive(label)` applies the stable
FNV-1a hash to the parent seed's little-endian bytes followed by the label;
`Seed::rng()` provides the repository's deterministic SplitMix64 generator.
Use named child seeds for independent choices such as resampling, feature
selection, or optimizer initialization instead of sharing a mutable stream.

Record the root seed, exact labels, and the order in which random choices are
made. The seed API does not log derivations, reserve label names, provide a
cryptographic random generator, or guarantee equivalence with external RNG
implementations.
