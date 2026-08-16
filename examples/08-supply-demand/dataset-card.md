# Dataset card: Supply and demand

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 401, sampled from `0` through `20` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `demand`, `supply`, `price`.
- **Known parameters:** `demand_rate`, `supply_rate`, `price_rate`, `target_price`, `cost`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
