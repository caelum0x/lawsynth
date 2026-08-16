# Noise handling

Deterministic world simulation has no injected noise. The only stochastic path is `euler_maruyama`, whose diffusion closure directly specifies independent per-coordinate noise amplitude and whose seed makes a path reproducible.

No observation-noise model, process-noise calibration, random-effects model, or ensemble uncertainty wrapper is added automatically.
