# Dataset card: Frictionless order book

- **Source:** deterministic numerical integration of the equations in `config.toml`.
- **Rows:** 1001, sampled from `0` through `20` at a `0.02` interval.
- **Time column:** `time`, strictly increasing floating-point simulation time.
- **State columns:** `mid`, `imbalance`.
- **Known parameters:** `impact`, `resilience` (`0`), `liquidity`.
- **Structure:** an undamped linear oscillator (conservative center at the
  origin) whose energy `liquidity·mid² + impact·imbalance²` is conserved to RK4
  truncation.
- **Use:** integration, finite-difference discovery, and — via `analyze.py` —
  fixed-point stability, invariant detection, and forecasting.
- **Limitations:** this is synthetic data with no measurement process, missingness,
  or external validity claim.
