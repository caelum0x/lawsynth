# expression-throughput

Executes candidate metric comparison and Pareto selection through the public Python SDK. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/expression-throughput/run.py.
