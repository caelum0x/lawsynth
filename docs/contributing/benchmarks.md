# Benchmark contributions

Benchmarks measure an implemented API under a named workload; they are not
performance claims by themselves. Place Rust microbenchmarks beside the crate
that owns the operation and use `black_box` around input/output to prevent
irrelevant optimization. Run them with the crate's declared bench target or
`cargo bench -p CRATE` when the complete target is available.

Record hardware, toolchain, input size, sampling characteristics, and the
command used before comparing runs. Use deterministic generated fixtures or
versioned datasets. A regression threshold needs a documented baseline and
noise policy, not a single local timing.

Benchmark folders for future causal, regime, uncertainty, service, or plugin
features may verify an explicit unsupported boundary; they must not present
synthetic timing numbers as if those runtimes existed.
