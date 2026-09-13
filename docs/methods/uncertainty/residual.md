---
title: "Residual uncertainty: limits and forecast bands"
description: Residual uncertainty is error left after prediction. Learn what LawSynth summarizes, what it does not infer, and how seeded bootstrap forecast bands work.
tags: residual uncertainty, prediction error, residual bootstrap, forecast bands
---

# Residual uncertainty: limits and forecast bands

**Residual uncertainty** describes the prediction error that remains after a model is fitted or evaluated. LawSynth defines each signed residual as `observed - predicted`, in the original observation order. A positive value means the observation is above the prediction; a negative value means it is below.

## What LawSynth computes

The `lawsynth-score` crate exposes the residual vector and three descriptive summaries:

- the mean residual, which can reveal systematic overprediction or underprediction;
- the population standard deviation, which describes the residual spread;
- the maximum absolute residual, which reports the largest observed miss.

These values describe a supplied set of errors. They do not fit a probability distribution, estimate observation noise, model autocorrelation, or account for changing variance over time. A small aggregate residual also does not prove that a discovered equation will forecast well outside the fitted interval. Use chronological holdout validation and inspect the residual sequence rather than relying on one statistic.

## Residual uncertainty versus forecast bands

A residual summary is not a prediction interval. LawSynth creates forecast bands only when you request the separate residual-bootstrap workflow and provide reference observations:

```bash
lawsynth forecast model.lsworld \
  --horizon 5 --start 0 --step 1 --initial x=10 --initial y=5 \
  --confidence --data observations.csv --time time \
  --level 0.90 --replicates 512 --seed 7
```

For each state, that workflow simulates the model over the observed window, forms residuals, resamples them with replacement, and applies the resulting offsets to the forecast. The seed makes repeated runs reproducible. The command reports the residual count, lower and upper offsets, and standard error used for each state.

## Assumptions and boundaries

Residual bootstrap treats the sampled errors as exchangeable. It does not preserve time blocks or explicitly represent heteroscedasticity and serial correlation. The resulting band is therefore an empirical sensitivity summary under that resampling assumption, not a guaranteed coverage statement, a causal confidence interval, or coefficient uncertainty.

Use coefficient bootstrap during discovery when the question concerns stability of fitted equation terms. Use residual-bootstrap forecast bands when the question concerns forecast variation relative to an observed reference window. If the residual structure changes across regimes or over time, use a procedure that models that structure explicitly instead of interpreting the default band as a complete noise model.

## Related guides

- [Validate and forecast a discovered world](../../guide/workflow.md)
- [Forecast confidence bands and what-if scenarios](../../cookbook/04-forecast-and-whatifs.md)
- [Empirical bootstrap boundary](bootstrap.md)
- [Coefficient uncertainty contract](https://github.com/caelum0x/lawsynth/tree/main/specs/coefficient-uncertainty)
