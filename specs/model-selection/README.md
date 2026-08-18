# Cross-validated model selection boundary (time-series CV)

This directory specifies the **deterministic cross-validated hyperparameter
selection** for discovery, implemented in `crates/lawsynth-modelselect`
(`select_model`, `sweep_degrees_thresholds`). It is a **boundary specification**
in the house style: it states what a conforming implementation MUST do, and —
crucially — what a selected model and its cross-validation score are and are
*not* allowed to claim.

## Motivation

Discovery has knobs: the candidate-library polynomial degree, the sparsity
threshold, the sparse solver. Set them by hand and you either underfit (a library
too small to hold the true law) or overfit (a library so large it fits the
derivative noise, then generalizes poorly). A scientific product should pick these
knobs *by measured out-of-sample behaviour*, not by taste.

The standard answer is cross-validation, but dynamics data is a time series:
shuffling rows into random folds leaks the future into the past and destroys the
serial structure the model exists to capture. So this crate uses **time-series
cross-validation** — contiguous, forward-in-time folds — and scores each
candidate by its **predictive generalization**: discover on a training segment,
then *re-simulate the discovered world* across a held-out future segment and
measure how well the simulated trajectory tracks the observations. That
re-simulation is the honest test of a governing equation: a law that only fits
the training derivatives but cannot forecast is not a law.

LawSynth is deterministic and offline. Fold boundaries, sweep order, discovery,
simulation, and scoring are all deterministic, so identical inputs MUST yield a
bit-identical report.

## What a CV score IS

A cross-validation score here is an **empirical estimate of out-of-sample
predictive skill**, never a guarantee. The contract is:

1. **Discover on train, simulate on test.** Each fold fits a model on its
   training segment only, then integrates that model forward across the held-out
   test segment from the test segment's first observed state, and scores the
   simulated trajectory against the observations. The training data never appears
   in the score.
2. **Reuse the shipped scoring.** Fit is measured with
   `lawsynth_score::fit_statistics` — the same R²/RMSE helper behind the CLI
   `validate` command — applied to the simulated trajectory interpolated onto the
   observed timestamps. The selection metric (`ScoreMetric::RSquared` or
   `ScoreMetric::Rmse`) is normalized so **higher is always better** (RMSE is
   negated).
3. **Full, auditable table.** The report lists *every* candidate's mean and
   per-fold scores, its swept-knob summary, its full-data active-term count, and
   its failed-fold count — not just the winner. Selection is reproducible and
   inspectable; `SelectionReport::render_table` prints it.
4. **Failures are recorded, not hidden.** A candidate whose discovery or
   simulation fails on a fold (resource limit, solver error, a diverged
   non-finite trajectory, an unscoreable constant segment) is recorded with a
   failing `FoldStatus` and the documented worst-case score (`-1.0e18`, a finite
   sentinel so means stay bit-reproducible). It is never silently dropped, and it
   sinks in the ranking rather than vanishing.

## Requirements

1. **No random shuffling.** Folds MUST be contiguous segments in time order. The
   timeline of `n` samples is cut into `folds + 1` near-equal segments at integer
   boundaries `floor(j·n / (folds+1))`; fold `i` always tests on segment `i+1`.
   Two schemes are offered:
   - **Forward chaining** (`CvScheme::ForwardChaining`): train on *all* samples
     before the test segment (segments `0..=i`). The training window grows; the
     model is always fit on the past and scored on the future — the standard
     time-series CV.
   - **Rolling blocks** (`CvScheme::RollingBlocks`): train on the single block
     immediately preceding the test segment. A fixed-width window that slides.
2. **Determinism.** Fold boundaries (integer arithmetic), sweep order (grid
   order), discovery, simulation, and score aggregation (fixed iteration order)
   MUST be deterministic. Identical `(Dataset, grid, CvConfig)` MUST produce a
   bit-identical `SelectionReport` (verified to `f64::to_bits`).
3. **Deterministic tie-break toward simpler models.** The winner is the maximum
   mean score. Ties MUST be broken by a fixed, documented order that prefers the
   simpler model: (1) higher mean score, (2) lower polynomial degree, (3) higher
   sparsity threshold, (4) fewer full-data active terms (`None` sorts as most
   complex), (5) lower grid index. This means that when a larger library prunes
   its spurious terms and generalizes identically to a smaller one, the smaller
   one is selected.
4. **Honest failure accounting.** A per-fold discovery/simulation/scoring failure
   MUST be recorded as a failing fold with the sentinel score and surfaced in
   `failed_folds`; it MUST NOT abort the sweep or be omitted from the mean.
5. **Validation at the boundary.** An empty grid, a zero fold count, and a dataset
   too short to split into the requested folds within the configured train/test
   sample floors MUST return a typed `ModelSelectError`, never a silent degenerate
   result.
6. **Offline, std-only.** No network, no external crates; internal path
   dependencies only (`lawsynth-discovery`, `lawsynth-sim`, `lawsynth-score`,
   `lawsynth-data`, `lawsynth-world`).

## Public API

```text
select_model(&Dataset, grid: &[DiscoveryConfig], &CvConfig)
    -> Result<SelectionReport, ModelSelectError>

sweep_degrees_thresholds(&Dataset, base: &DiscoveryConfig,
    degrees: &[usize], thresholds: &[f64], &CvConfig)
    -> Result<SelectionReport, ModelSelectError>
```

`CvConfig` carries the fold-assignment `scheme`, the `folds` count, the selection
`metric`, and the per-fold `min_train_samples` / `min_test_samples` floors.
`SelectionReport` lists, per candidate (`CandidateScore`): the swept-knob
`ConfigSummary`, `mean_score`, per-fold `FoldScore`s (train/test ranges, status,
R², RMSE, selection score), `failed_folds`, and full-data `active_terms`, plus the
`best_index`. `sweep_degrees_thresholds` is a convenience that builds the
degree×threshold grid (degrees outer, thresholds inner) from a base config.

## Honest scope & limits

- **A CV score is an estimate, not a guarantee.** It estimates generalization on
  data drawn like the held-out folds; it does not certify behaviour on genuinely
  new regimes, and a percentile of one dataset is not a coverage bound.
- **Forward chaining assumes stationarity across folds.** If the dynamics drift
  between the early training segments and the late test segments (regime change,
  non-stationarity), the CV estimate is biased and the selected knobs may not be
  best for the future.
- **Simulation-based scoring inherits the integrator's error and penalizes
  chaos.** The score is computed by integrating the discovered world with the
  fixed-step RK4 in `lawsynth-sim`. For chaotic systems, two trajectories with
  *identical* governing equations diverge exponentially, so even a correctly
  recovered law can earn a poor held-out score once the test window outruns the
  Lyapunov horizon. A low CV score on chaotic data is evidence about
  *predictability*, not necessarily about model correctness — read it accordingly.
- **Exogenous inputs are held constant.** Non-state (control) columns are held at
  their test-origin value during re-simulation; a candidate driven by a fast-
  varying exogenous input will be scored conservatively.
- **Selection is only as good as the grid.** The grid is user-supplied. The sweep
  finds the best *offered* candidate, not the best possible model; a poorly chosen
  grid yields a poorly chosen model. The full score table is provided so the user
  can see whether the grid bracketed a sensible optimum or should be widened.

## Non-goals

- No random or stratified k-fold, no wall-clock-seeded shuffling.
- No nested CV, no automatic grid refinement, no Bayesian optimization: the grid
  is supplied and evaluated as given.
- No claim of guaranteed generalization: a selected model is the best-scoring
  candidate under a reproducible, auditable predictive test, nothing more.
