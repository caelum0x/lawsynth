# LawSynth plugin API

`lawsynth-plugin-api` is the portable extension contract. It has no Rust ABI
promise: plugins communicate through a bounded binary frame protocol and declare
their identity, requested capabilities, execution kind, and resource budget.

The manifest parser deliberately accepts a small, deterministic `key = value`
format. It rejects unknown keys, unsafe entrypoints, duplicate fields, malformed
versions, and capability names it does not understand. Hosts must still grant
each declared permission explicitly.

Use `Frame::encode` and `Frame::decode` for process or WASI transport. Validate
algorithm, adapter, and simulator inputs before accepting plugin output.
