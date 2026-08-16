# parquet-load

Runs the Rust data crate's uncompressed PLAIN Parquet decoding contract. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/parquet-load/run.py.
