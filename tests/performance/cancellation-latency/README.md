# cancellation-latency

Runs the native discovery cancellation contract; it verifies a pre-cancelled request terminates through the real engine. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/cancellation-latency/run.py.
