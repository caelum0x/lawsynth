# Diagnostics and failures

Simulation validates finite time grids, inputs, and overrides; rejects missing or unknown state values; and rejects non-finite expression results. It detects floating-point time-resolution loss when a requested step cannot advance the clock.

The returned trajectory contains sample times and per-state series. It does not include local truncation error, conserved-quantity drift, stiffness diagnostics, or a solver convergence certificate.
