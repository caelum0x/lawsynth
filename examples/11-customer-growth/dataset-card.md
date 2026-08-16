# Dataset card: Customer growth

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 201, sampled from `0` through `10` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `customers`.
- **Known parameters:** `r`, `capacity`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
