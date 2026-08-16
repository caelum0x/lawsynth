# Dataset card: Lorenz attractor

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 401, sampled from `0` through `4` at a `0.01` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `x`, `y`, `z`.
- **Known parameters:** `sigma`, `rho`, `beta`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
