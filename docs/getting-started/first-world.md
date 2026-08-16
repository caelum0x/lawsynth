# Your first World

Use the quickstart's CSV route when the law should be inferred from
observations. The generated World is a bundle, not a generic JSON document:
the Rust bundle writer chooses its canonical archive layout and validates the
World before writing it.

Inspect a generated artifact before using it in another workflow:

```sh
cargo run -p lawsynth-cli -- inspect decay.lsworld
```

The inspector distinguishes continuous and discrete Worlds and reports state,
variable, and parameter counts. A failed inspection means the artifact is not
accepted by either implemented bundle reader; do not treat a file extension as
proof of validity.

For a manually constructed World, use the Rust `lawsynth-world` API or the
built Python native API. Keep source values finite, identifiers valid, units
compatible, and exactly one law assigned to each state.
