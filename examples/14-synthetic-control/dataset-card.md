# Dataset card: Synthetic control

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 241, sampled from `0` through `12` at a `0.05` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `treated`, `donor`.
- **Known parameters:** `coupling`, `donor_growth`, `effect`, `treatment_time`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
