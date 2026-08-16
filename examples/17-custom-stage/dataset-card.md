# Dataset card: Custom discovery stage

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 161, sampled from `0` through `8` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `signal`.
- **Known parameters:** `decay`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
