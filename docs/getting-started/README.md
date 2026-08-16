# Getting started

LawSynth's executable local path is Rust-first: numeric CSV observations are
turned into a scalar continuous World, persisted as a deterministic
`.lsworld` bundle, and simulated locally. The public process interface is the
`lawsynth` CLI; Python wraps the same native implementation.

Start with [installation](installation.md), then run the
[quickstart](quickstart.md). Use [concepts](concepts.md) before supplying a
dataset, and use the [CLI](cli.md) or [Python](python.md) pages when choosing
an interface.

The checked-in examples and tests are executable. Planned server, plugin,
Studio, causal, regime, and uncertainty directories do not extend the current
runtime contract.
