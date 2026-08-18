# Stochastic (SDE) law discovery boundary (v2-A)

This directory specifies stochastic law discovery — the Kramers–Moyal estimator
implemented in `crates/lawsynth-sde`, which recovers the **drift** `a(x)` and
**diffusion** `b²(x)` of a diagonal-noise Itô SDE `dX = a(X) dt + b(X) dW` from
noisy sample paths. It is a **boundary specification** in the house style: it
states what a conforming implementation MUST do, and is explicit about the
statistical nature of the result.

It closes the loop with the forward direction: `lawsynth-sim::euler_maruyama`
*simulates* such an SDE; this crate *discovers* one from data.

## Method

For a time step `Δt`, the conditional moments of the increment
`ΔX = X(t+Δt) − X(t)` given `X(t) = x` estimate the first two Kramers–Moyal
coefficients:

```text
a(x)  ≈ E[ΔX  | X = x] / Δt          (drift)
b²(x) ≈ E[ΔX² | X = x] / Δt          (diffusion; the drift² term is higher order in Δt)
```

The conditional expectations are estimated by **binning** the observed state
space and averaging `ΔX/Δt` and `ΔX²/Δt` within each bin. Bins with enough
occupancy (the *trusted* bins) are then **sparse-regressed** onto a polynomial
candidate library to yield closed-form laws for drift and diffusion. Both the
raw binned table and the fitted laws MUST be reported.

## Requirements

1. **Kramers–Moyal estimator.** The drift MUST be the first conditional moment of
   the increment divided by `Δt`, and the diffusion the second — never a
   finite-difference derivative of the (noisy) path. `Δt` MUST come from the
   dataset's time axis, using the per-step spacing so irregular-but-declared
   grids are honoured (or rejected — see 4).
2. **Occupancy gating.** Each reported bin MUST carry its sample `count`, and the
   sparse fit MUST use only bins whose occupancy meets a configured
   `min_bin_count`. The number of `trusted_bins` MUST be reported so a caller can
   judge whether the state space was adequately sampled. Rarely-visited bins MUST
   NOT be silently trusted.
3. **Both raw and fitted output.** The result MUST expose the per-bin
   `(x_center, drift, diffusion, count)` table *and* the sparse-regressed drift
   and diffusion laws (labeled polynomial terms + residual). A caller MUST be
   able to inspect the estimator's raw evidence, not only the fitted summary.
4. **Determinism.** Binning, averaging, library evaluation, and the sparse solve
   MUST run in a fixed order with no hidden randomness. Any synthetic path used to
   validate the method MUST be generated from a seeded generator (never the wall
   clock). Identical `(Dataset, SdeConfig)` inputs MUST yield a bit-identical
   `SdeModel`.
5. **Degenerate input.** A state with no spread (no finite-width bins definable),
   an irregular time axis when regular spacing is required, or a path too short to
   form increments MUST return a typed error, never a fabricated law.

## Honest limits

This is a **statistical estimator**, and the specification is deliberate about
what that does and does not guarantee:

- Accuracy depends on **path length**, on `Δt` being small enough for the
  Kramers–Moyal expansion to hold yet large enough to average out sampling noise,
  and on **bin occupancy**. Longer paths tighten the estimate; a conforming
  implementation MUST NOT claim machine-precision recovery of a stochastic
  coefficient. The reference tests assert recovery only to tolerances appropriate
  to the path length and bin count they use.
- The finite-`Δt` bias of the Kramers–Moyal coefficients is real: the estimates
  are exact only in the `Δt → 0` limit.
- The method assumes a **Markovian Itô SDE with diagonal (per-state) noise** and a
  well-sampled state space. Multiplicative-noise subtleties, the Itô/Stratonovich
  distinction, correlated/cross-state noise, and jump processes are **out of
  scope** for this boundary; they may be added as extensions with their own
  contracts.
- Boundaries of the observed range and low-occupancy bins are the least reliable
  and are gated, not extrapolated.
