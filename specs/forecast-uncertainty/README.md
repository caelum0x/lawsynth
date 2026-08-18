# Forecast uncertainty boundary (delta method + seeded Monte-Carlo)

This directory specifies **deterministic propagation of coefficient uncertainty
into forecast intervals** — mapping a parameter covariance `Cov(θ)` onto
trajectory prediction bands for a discovered model, implemented in
`crates/lawsynth-propagate` (`delta_forecast`, `monte_carlo_forecast`,
`covariance_from_ensemble`). It is a **boundary specification** in the house
style: it states what a conforming implementation MUST do, and — crucially —
what a reported band is and is *not* allowed to claim.

## Motivation

A discovered dynamical law `ẋ = f(x; θ)` is only half the story. The coefficients
`θ` are themselves estimates: the deterministic bootstrap of
`crates/lawsynth-uncertainty` returns an ensemble of re-fit coefficient vectors,
whose spread is the parameter covariance `Cov(θ)`. A forecast that ignores that
spread is dishonest — it draws a single line where the data support a band.

This crate closes the loop: it propagates **parameter** uncertainty into
**trajectory** uncertainty, so `x(t)` arrives with a prediction band. It offers
two methods that MUST agree in the small-uncertainty limit and are allowed —
indeed required — to diverge honestly outside it:

- **Delta method (analytic, first-order).** The forward sensitivities
  `S(t) = ∂x(t)/∂θ` (from `crates/lawsynth-sensitivity`) are the linear map from
  parameter space to state space. To first order,
  `Cov(x(t)) ≈ S(t)·Cov(θ)·S(t)ᵀ`; the per-state variance is that product's
  diagonal and the band is `x(t) ± z·sqrt(variance)`.
- **Monte-Carlo (seeded ensemble).** Draw `M` parameter vectors from the
  ensemble, simulate each with the *same* fixed-step RK4 the sensitivities use,
  and take the per-time empirical mean and percentile band.

LawSynth is **deterministic and offline**. The bands are computed with analytic
sensitivities and a seeded pseudo-random generator — never a wall-clock sampler.
Identical inputs MUST yield bit-identical bands.

## What a band IS

A band here is a **quantified image of `Cov(θ)` under the model**, never a
coverage proof. The contract is:

1. **Delta = first-order linearization via sensitivities.** The delta variance is
   exactly `diag(S(t)·Cov(θ)·S(t)ᵀ)` with `S(t)` the forward-integrated
   sensitivities of the *reported discrete trajectory* (same RK4, same step). The
   band `x(t) ± z·sqrt(diag)` is a symmetric Gaussian band about the nominal
   trajectory `x(t)` integrated at the supplied `θ`.
2. **Monte-Carlo = seeded ensemble RK4.** `M` parameter vectors are drawn either
   by resampling the raw bootstrap replicate coefficient vectors with
   replacement, or by sampling `N(mean, Cov(θ))` via a deterministic
   SplitMix64 + Box–Muller draw shaped by the Cholesky factor of `Cov(θ)`. Each
   vector is simulated with the same fixed-step RK4, and the band is the per-time
   empirical mean, unbiased variance, and two-sided percentile interval.
3. **`Cov(θ)` comes from the bootstrap replicates.** `covariance_from_ensemble`
   is the unbiased (`B − 1` denominator) sample covariance of the replicate
   coefficient vectors, matching the unbiased per-term `standard_error` the
   uncertainty crate already reports. This is the intended pipeline: bootstrap →
   covariance → band.
4. **Honest reporting, no fabrication.** The propagator reports mean, variance,
   and band endpoints as computed. It never invents a distribution, never
   silently regularizes a bad covariance, and never suppresses a legitimate zero
   (a parameter absent from the fields has zero sensitivity, hence contributes
   zero variance — reported as such).

## The two maps

For state `x ∈ Rⁿ`, parameters `θ ∈ Rᵖ`, covariance `Σ_θ = Cov(θ)`, and forward
sensitivities `S(t) = ∂x(t)/∂θ` (an `n × p` matrix):

```text
Delta:        Cov(x(t)) ≈ S(t)·Σ_θ·S(t)ᵀ
              Var(x_i(t)) = Σ_j Σ_l S_ij(t)·Σ_θ[j][l]·S_il(t)
              band_i(t)   = x_i(t) ± z·sqrt(Var(x_i(t)))

Monte-Carlo:  θ^(m) ~ ensemble           (m = 1 … M, seeded by (seed, m))
              x^(m)(t) = RK4-simulate(f, x0, θ^(m))
              mean_i(t)  = (1/M) Σ_m x_i^(m)(t)
              band_i(t)  = [ Q_{α/2}, Q_{1−α/2} ] of { x_i^(m)(t) }_m
```

with `α = 1 − confidence` and quantiles by the R type-7 rule (the same rule as
`lawsynth-uncertainty::percentile`). The two maps coincide as `Σ_θ → 0`.

## Requirements

1. **Delta from analytic sensitivities.** A conforming implementation MUST obtain
   `S(t)` from `crates/lawsynth-sensitivity`'s `forward_sensitivities` (analytic
   Jacobian and analytic parameter partials, one shared fixed-step RK4). The
   nominal trajectory reported as the band centre MUST be the state block of that
   same integration, so the sensitivities are the sensitivities of the *reported*
   trajectory. It MUST NOT finite-difference the field.

2. **Monte-Carlo mirrors the same integrator.** Each drawn parameter vector MUST
   be simulated with the same fixed-step RK4 (same `t0`, `dt`, `steps`) as the
   sensitivities, so that as `Σ_θ → 0` the Monte-Carlo mean converges to the
   delta nominal trajectory. The two methods MUST share the time grid.

3. **`Cov(θ)` from the bootstrap replicates.** `covariance_from_ensemble` MUST be
   the unbiased sample covariance (`B − 1` denominator) of the replicate
   coefficient vectors, so it is consistent with the crate's per-term
   `standard_error`. Callers MAY also supply a covariance directly (delta and the
   Gaussian Monte-Carlo source) or the raw replicates (the resampling
   Monte-Carlo source).

4. **Determinism.** All arithmetic (the quadratic form `S·Σ·Sᵀ`, the Cholesky
   factor, the RNG, the empirical aggregation) MUST be accumulated in a fixed
   order. The Monte-Carlo generator MUST be seeded per sample from `(seed, m)`
   using the same SplitMix64 family as the coefficient bootstrap, so the ensemble
   is **independent of iteration order** and bit-reproducible. Identical inputs
   MUST produce a **bit-identical** `ForecastBands` (verified to `f64::to_bits`
   via a canonical-string fingerprint).

5. **Totality and typed errors.** The implementation MUST reject, with distinct
   typed errors and never a fabricated or silently dropped band:
   - a covariance that is not square, or whose dimension differs from the
     parameter count;
   - a non-finite covariance, mean, or replicate value, or a non-finite band
     multiplier `z`;
   - an indefinite covariance — both a Cholesky failure (Monte-Carlo Gaussian
     draw) and a meaningfully negative delta variance MUST surface as
     "not positive semi-definite" rather than a `NaN` or complex result;
   - a Monte-Carlo request for zero samples, a confidence outside `(0, 1)`, an
     empty replicate ensemble, or a replicate of the wrong width;
   - any structural or numeric failure surfaced by the underlying sensitivity
     integration (unknown symbol, dimension mismatch, invalid config, …), passed
     through verbatim.
   A tiny negative delta variance produced by roundoff from a legitimately
   positive-semi-definite covariance is clamped to zero, not reported as an error.

6. **Query surface.** `ForecastBands` MUST expose the shared time grid, the state
   ordering, and the four `[state][time]` matrices (mean, variance, lower, upper),
   plus a band-width helper (out-of-range indices return `None`, never panic) and
   a canonical-string fingerprint for determinism checks.

7. **Honest verification.** Any claim that the bands are correct MUST be backed by
   reproducible cross-checks: (a) an **analytic check** — for `ẋ = −θ·x` with
   `Var(θ) = s²`, the delta variance is exactly `(t·x0·e^{−θt})²·s²`, checked to a
   tight tolerance; (b) **delta ≈ Monte-Carlo for small uncertainty** on a
   nonlinear model (logistic), agreeing to a stated relative tolerance, together
   with a demonstration that they **diverge** under large uncertainty (the honest
   first-order limitation); (c) **monotonicity** (bands widen with time for a
   growing sensitivity, and with larger `Cov(θ)`); (d) **coverage sanity** (the
   nominal trajectory lies inside the Monte-Carlo band; higher confidence widens
   it); (e) an **end-to-end** run from a real bootstrap ensemble through
   `covariance_from_ensemble` to bands; and (f) **determinism** (bit-identical
   bands). The reference suite pins the analytic check to `1e-9` and the
   delta/Monte-Carlo variance agreement to 10% relative at small `σ`.

## Public API

```text
delta_forecast(
    &[(Identifier, Expr)],   // fields  ẋ_i = f_i(x; θ)
    &[Identifier],           // states  (output ordering)
    &[Identifier],           // parameters
    &[f64],                  // initial state x(t0)
    &[f64],                  // nominal parameter values θ
    &[Vec<f64>],             // Cov(θ), p × p
    &SensitivityConfig,
    f64,                     // band multiplier z
) -> Result<ForecastBands, PropagateError>

monte_carlo_forecast(
    &[(Identifier, Expr)], &[Identifier], &[Identifier], &[f64],
    EnsembleSource,          // Gaussian { mean, covariance } | Replicates { draws }
    &SensitivityConfig,
    usize,                   // samples M
    u64,                     // seed
    f64,                     // confidence in (0, 1)
) -> Result<ForecastBands, PropagateError>

covariance_from_ensemble(&CoefficientEnsemble) -> Vec<Vec<f64>>   // unbiased sample Cov(θ)
z_for_confidence(f64) -> Result<f64, PropagateError>              // two-sided normal multiplier

ForecastBands::times() -> &[f64]
ForecastBands::states() -> &[Identifier]
ForecastBands::mean() / variance() / lower() / upper() -> &[Vec<f64>]   // [state][time]
ForecastBands::band_width(state, time) -> Option<f64>
ForecastBands::to_canonical_string() -> String                          // determinism fingerprint
```

`z_for_confidence` maps a two-sided confidence level to the delta multiplier via
the standard normal quantile (Acklam's rational approximation), so the delta band
and a Monte-Carlo percentile band can be compared on the same footing.

## Honest scope & limits

- **The delta method is FIRST-ORDER.** `S(t)·Cov(θ)·S(t)ᵀ` is the exact
  covariance of the *linearized* flow. Under strong model nonlinearity or large
  parameter uncertainty the true `x(t)` distribution is skewed, so the symmetric
  delta band **undercovers** and drifts from the Monte-Carlo band. The reference
  suite exercises exactly this divergence — it is a property, not a bug.
- **Monte-Carlo coverage is only as good as the ensemble.** The `Replicates`
  source inherits every limitation of the bootstrap (percentile intervals can
  undercover under misspecification; case resampling assumes exchangeable rows).
  The `Gaussian` source additionally assumes the coefficients are approximately
  normal with the given mean and covariance — a modelling choice, not a theorem.
- **Gaussian bands assume approximate normality of `x(t)`.** The delta band
  reports `mean ± z·sd`; reading `z` as a coverage level assumes `x(t)` is
  approximately normal, which the nonlinear flow only guarantees in the
  small-uncertainty limit.
- **Both inherit upstream biases.** The sensitivities and the RK4 re-simulations
  carry the fixed-step integrator's `O(dt⁴)` error and drift over long horizons;
  and neither method corrects for the finite-difference `ẋ` bias baked into the
  original coefficient fit. A band is honest about the discretisation and the
  covariance it was handed, not about the true continuous flow or the true data.
- **`Cov(θ)` must be positive semi-definite.** An indefinite covariance has no
  Cholesky factor and yields negative variances; the propagator reports this
  rather than regularizing it away. A near-singular covariance (e.g. a parameter
  absent from the fields, giving a zero row) is valid for the delta method but not
  for the Gaussian Monte-Carlo draw — use the `Replicates` source or the delta
  method there.

## Non-goals

- No second-order (Hessian) delta correction, no unscented transform, and no
  adjoint/reverse propagation — a single first-order forward map only.
- No adaptive sampling, importance sampling, or variance reduction — a flat,
  seeded ensemble only.
- No coverage calibration or conformal band — the bands are the honest image of
  `Cov(θ)`, not a guaranteed-coverage interval.
- No uncertainty in the initial condition, the model structure, or the time grid;
  only coefficient uncertainty is propagated.
- No stochastic (wall-clock-seeded) sampling, no network, no external crates.
