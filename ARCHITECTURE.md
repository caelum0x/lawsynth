# Architecture

LawSynth is a Rust-first local engine. `lawsynth-core` supplies common
validation, diagnostics, hashing, cancellation, and resource primitives.
World, expression, unit, data, simulation, discovery, and bundle crates build
the implemented execution path; `lawsynth-cli` exposes it as a process API and
`lawsynth-python` exposes selected native capabilities to Python.

The principal data flow is:

```text
numeric CSV -> data/profile/derivatives/features -> sparse discovery
           -> World IR -> canonical .lsworld bundle -> simulation/inspection
```

`LawSynth_Production_Architecture.md` records the broader intended system.
Directories for later services, UI, plugins, causal inference, regimes, and
uncertainty are not evidence of implemented runtime behavior. The crate APIs,
CLI help, and executable tests define current support.
