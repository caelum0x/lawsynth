# ode-simulation

Executes a real native RK4 trajectory from a validated stored .lsworld bundle. It is an executable performance contract, not a benchmark fixture: run.py performs the named operation, validates its observable result, and enforces the portable wall-clock budget declared in case.toml.

Run it from the repository root with python3 tests/performance/ode-simulation/run.py.
