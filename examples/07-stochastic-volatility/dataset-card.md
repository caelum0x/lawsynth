# Dataset card: Stochastic volatility

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 201, sampled from `0` through `10` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `price`, `variance`.
- **Known parameters:** `drift`, `mean_reversion`, `long_variance`, `vol_of_vol`, `seed`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
