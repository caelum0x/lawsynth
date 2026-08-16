# Dataset card: Energy load

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 241, sampled from `0` through `24` at a `0.1` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `load`.
- **Known parameters:** `base_load`, `amplitude`, `period`, `relaxation`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
