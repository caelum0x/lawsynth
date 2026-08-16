# studio-paint

Measures supported render-host payload generation; the studio itself is intentionally not part of the P1 runtime. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/studio-paint/run.py.
