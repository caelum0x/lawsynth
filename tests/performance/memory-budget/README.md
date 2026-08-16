# memory-budget

Allocates and validates a substantial numeric Dataset through the Python SDK while tracing peak allocation. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/memory-budget/run.py.
