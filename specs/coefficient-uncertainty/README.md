# Coefficient uncertainty boundary (deterministic bootstrap)

This directory specifies the **deterministic bootstrap / ensemble uncertainty**
for discovered sparse coefficients, implemented in `crates/lawsynth-uncertainty`
(`bootstrap_coefficients`, `CoefficientEnsemble`). It is a **boundary
specification** in the house style: it states what a conforming implementation
MUST do, and — crucially — what a reported interval or inclusion probability is
and is *not* allowed to claim.

## Motivation

Sparse regression (STLSQ and friends in `crates/lawsynth-sparse`) returns a
single coefficient vector: a point estimate of a candidate law. A scientific
discovery product must also answer *how sure are we?* — for each candidate term,
how much would the coefficient move under a different draw of the data, and how
often does the term survive sparsity at all. That last quantity, the
**inclusion probability**, is the honest headline signal: a term recovered in
2% of resamples is noise dressed as structure, regardless of how clean its
single point estimate looks.

LawSynth is deterministic and offline, so the uncertainty machinery is a
**seeded, reproducible resampling ensemble**, not a stochastic sampler drawing
from the wall clock. Identical inputs MUST yield a bit-identical ensemble.

## What a reported interval IS

A bootstrap confidence interval here is an **empirical percentile interval over
re-fits**, never a proof of coverage. The contract is:

1. **Resample, re-fit, aggregate.** The ensemble draws `B` resamples of the
   rows, re-fits the sparse solver (STLSQ) on each, and aggregates the resulting
   coefficient vectors column-by-column.
2. **Percentile method.** The interval for a coefficient is the `α/2` and
   `1 − α/2` empirical quantiles of its bootstrap distribution, with
   `α = 1 − confidence`. Quantiles use linear interpolation between the two
   nearest order statistics at rank `p·(n − 1)` (the R type-7 rule) over values
   placed in a total ordering — the same deterministic rule as
   `crate::percentile`.
3. **Inclusion probability is the headline.** For each candidate term the
   ensemble reports the fraction of resamples in which the coefficient stayed
   non-zero (survived sparsity). STLSQ writes an exact `0.0` for pruned terms, so
   this is an exact-zero test, not a fuzzy threshold on the aggregate.
4. **Honest reporting, no fabrication.** The ensemble reports mean, standard
   error (the spread of the bootstrap distribution, not the standard error of the
   mean), interval endpoints, and inclusion probability. It never imputes,
   discards, or manufactures a distribution.

## Requirements

1. **Determinism.** Resample `b` draws its indices from a SplitMix64 state
   derived solely from `(seed, b)` (reusing the crate's `next_u64` / `next_index`,
   the same SplitMix64 as `lawsynth_core::DeterministicRng`). Because each
   replicate is a pure function of `(seed, b)`, the ensemble is bit-reproducible
   and independent of iteration order or thread count. Identical
   `(theta, target, config)` MUST produce a bit-identical `CoefficientEnsemble`
   (verified to `f64::to_bits`), and a prefix of the replicate draws MUST be
   stable as `B` grows.
2. **Two resampling modes, both deterministic.**
   - **Case / pairs bootstrap** (`ResampleMode::Cases`): resample
     `(row_of_Θ, target)` pairs with replacement. Assumes exchangeable rows.
   - **Residual bootstrap** (`ResampleMode::Residual`): fit once, resample the
     fitted residuals with replacement, and rebuild synthetic targets `ŷ + r*`
     with the design matrix held fixed. Assumes homoscedastic, exchangeable
     residuals.
3. **Real re-fits.** Every replicate MUST be re-fit with the configured sparse
   solver (STLSQ) and the configured threshold/ridge/iterations; the ensemble
   never re-uses the base fit as a stand-in for a resample.
4. **Validation at the boundary.** The design matrix and target MUST be
   dimension-checked and finite before any fitting; `B < 2`,
   `confidence ∉ (0, 1)`, dimension mismatch, and non-finite inputs MUST return a
   typed `UncertaintyError`, never a silent degenerate result. A degenerate
   all-zero target is a valid input and MUST yield an all-zero ensemble with zero
   inclusion — not an error and not a fabricated interval.
5. **Offline, std-only.** No network, no external crates; internal path
   dependencies only (`lawsynth-sparse` for the re-fit).

## Public API

```text
bootstrap_coefficients(theta: &[Vec<f64>], target: &[f64], &BootstrapCoefficientConfig)
    -> Result<CoefficientEnsemble, UncertaintyError>
```

`BootstrapCoefficientConfig` carries the resample count `B`, a fixed `seed`, the
`confidence` level, the `ResampleMode`, and the `SparseConfig` (threshold, ridge,
iterations) applied to every re-fit. `CoefficientEnsemble` reports, per candidate
term (`TermUncertainty`): `mean`, `standard_error`, `lower`/`upper` interval
endpoints, and `inclusion_probability`, plus the raw `replicates` draws for
transparency.

## Honest scope & limits

- **Bootstrap intervals are approximate.** Percentile intervals can *undercover*
  under model misspecification or heavy noise; treat them as indicative, not
  guaranteed-coverage.
- **Inclusion probability depends on the sparsity threshold.** It measures how
  often a term survives *this* STLSQ threshold, not a threshold-free truth. A
  different threshold yields a different inclusion probability, by design.
- **Case resampling assumes exchangeable rows.** Time-correlated dynamics data
  violates exchangeability, so intervals on trajectory-derived designs are
  indicative only. **Block resampling** for serially correlated rows is future
  work.
- **Not a posterior.** This is a frequentist resampling ensemble, not Bayesian
  inference; it fits no prior and returns no posterior density.

## Non-goals

- No stochastic (wall-clock-seeded) sampling, no network, no platform service.
- No block / stationary bootstrap for correlated data (future work).
- No claim of exact coverage: an interval is a reproducible, quantified estimate
  of coefficient stability, and inclusion probability is a reproducible estimate
  of term survival — both honest signals, neither a proof.
