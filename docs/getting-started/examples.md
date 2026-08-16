# Executable examples

The `examples/` directory contains reproducible workflows. Run its tests with
the import mode that keeps similarly named example modules isolated:

```sh
python3 -m pytest -q examples --import-mode=importlib
```

Each scenario produces its own data and validates the resulting behavior.
Supported deterministic numerical paths include the quickstart, Lorenz,
Lotka–Volterra, SIR, inventory, and bundle-interchange workflows. Examples
that name future capabilities—such as stochastic volatility, regime
switching, delayed feedback, custom stages, custom operators, or server
APIs—are capability-boundary exercises. They do not manufacture successful
results for features the production core cannot encode or execute.

Read each example's `README.md` and assertions before interpreting numerical
output as a scientific conclusion.
