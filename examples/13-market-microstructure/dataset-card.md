# Dataset card: Market microstructure

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 401, sampled from `0` through `20` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `midprice`, `imbalance`.
- **Known parameters:** `impact`, `resilience`, `liquidity`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
