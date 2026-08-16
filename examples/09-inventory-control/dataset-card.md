# Dataset card: Inventory control

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 51, sampled from `0` through `50` at a `1` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `inventory`, `backlog`.
- **Known parameters:** `target_inventory`, `demand`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
