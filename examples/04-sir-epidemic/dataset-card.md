# Dataset card: SIR epidemic

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 321, sampled from `0` through `80` at a `0.25` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `susceptible`, `infected`, `recovered`.
- **Known parameters:** `beta`, `gamma`, `population`.
- **Use:** integration, finite-difference discovery, and trajectory regression tests.
- **Limitations:** this is synthetic data with no measurement process, missingness, or external validity claim.
