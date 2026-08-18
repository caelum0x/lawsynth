//! Predictive, simulation-based scoring of a discovered world on a held-out
//! time segment.
//!
//! This mirrors `lawsynth-cli`'s `validate` command: simulate the discovered
//! world forward across the test window from the test segment's first observed
//! state, interpolate the trajectory onto the observed timestamps, and score fit
//! against the observations with [`lawsynth_score::fit_statistics`] (the reused
//! R²/RMSE helper). Exogenous (non-state) inputs are held constant at their
//! test-origin value, matching the forecast-origin convention.

use std::ops::Range;

use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_score::fit_statistics;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{VariableRole, World};

use crate::{FoldStatus, ModelSelectError, ScoreMetric};

/// Selection-driving score assigned to a fold that could not be scored (worst
/// possible finite, higher-is-better value). Kept finite so means stay
/// bit-reproducible while any failing candidate sinks to the bottom.
pub(crate) const FAILURE_SCORE: f64 = -1.0e18;

/// Outcome of scoring one fold: the honest per-metric measurements plus the
/// higher-is-better selection score derived from them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct FoldOutcome {
    pub status: FoldStatus,
    pub r_squared: Option<f64>,
    pub rmse: Option<f64>,
    pub score: f64,
}

impl FoldOutcome {
    fn failed(status: FoldStatus) -> Self {
        Self { status, r_squared: None, rmse: None, score: FAILURE_SCORE }
    }
}

/// Builds an owned sub-dataset from a contiguous index range of `dataset`.
///
/// The slice preserves strictly increasing timestamps and column order, so
/// `Dataset::new` validation always succeeds for a valid parent; any failure is
/// surfaced as [`ModelSelectError::Data`].
pub(crate) fn slice_dataset(
    dataset: &Dataset,
    range: Range<usize>,
) -> Result<Dataset, ModelSelectError> {
    let time = TimeAxis::new(dataset.time().values()[range.clone()].to_vec())
        .map_err(|error| ModelSelectError::Data(error.to_string()))?;
    let columns = dataset.columns().values().map(|column| {
        let sliced = NumericColumn::new(column.id.clone(), column.values[range.clone()].to_vec());
        match &column.unit {
            Some(unit) => sliced.with_unit(unit.clone()),
            None => sliced,
        }
    });
    Dataset::new(time, columns).map_err(|error| ModelSelectError::Data(error.to_string()))
}

/// Scores `world` against the observations in `test` under `metric`.
///
/// Never returns `Err`: any failure to simulate or score becomes a [`FoldOutcome`]
/// with a failing [`FoldStatus`] and the [`FAILURE_SCORE`], so a candidate that
/// breaks on a fold is honestly recorded rather than dropped.
pub(crate) fn score_world_on_segment(
    world: &World,
    test: &Dataset,
    metric: ScoreMetric,
) -> FoldOutcome {
    let times = test.time().values();
    if times.len() < 2 {
        return FoldOutcome::failed(FoldStatus::ScoringFailed);
    }
    let start = times[0];
    let end = times[times.len() - 1];
    let spacing = times[1] - times[0];
    let step = if spacing.is_finite() && spacing > 0.0 { spacing } else { 1.0 };
    let Ok(config) = SimulationConfig::new(start, end, step) else {
        return FoldOutcome::failed(FoldStatus::ScoringFailed);
    };

    // Forecast origin: observed state at the first test timestamp, plus any
    // exogenous inputs held constant at that same origin.
    let mut request = SimulationRequest::default();
    let state_ids: Vec<_> = world.state_ids().cloned().collect();
    for state in &state_ids {
        let Some(column) = test.columns().get(state) else {
            return FoldOutcome::failed(FoldStatus::ScoringFailed);
        };
        request = request.with_initial(state.clone(), column.values[0]);
    }
    for (id, variable) in world.variables() {
        if variable.role != VariableRole::State
            && let Some(column) = test.columns().get(id)
        {
            request = request.with_input(id.clone(), column.values[0]);
        }
    }

    let Ok(trajectory) = simulate(world, config, &request) else {
        return FoldOutcome::failed(FoldStatus::SimulationFailed);
    };

    // Aggregate per-state R² (only where the observed series has variance) and
    // RMSE (always) over the held-out window.
    let mut r_squared_sum = 0.0;
    let mut r_squared_count = 0usize;
    let mut rmse_sum = 0.0;
    let mut rmse_count = 0usize;
    for state in &state_ids {
        let Some(simulated) = trajectory.values.get(state) else {
            return FoldOutcome::failed(FoldStatus::ScoringFailed);
        };
        let observed = &test.columns()[state].values;
        let predicted = interpolate_onto(&trajectory.time, simulated, times);
        let Ok(stats) = fit_statistics(observed, &predicted) else {
            return FoldOutcome::failed(FoldStatus::ScoringFailed);
        };
        rmse_sum += stats.root_mean_squared_error;
        rmse_count += 1;
        if has_variance(observed) {
            r_squared_sum += stats.r_squared;
            r_squared_count += 1;
        }
    }

    if rmse_count == 0 {
        return FoldOutcome::failed(FoldStatus::ScoringFailed);
    }
    let mean_rmse = rmse_sum / rmse_count as f64;
    let mean_r_squared = (r_squared_count > 0).then(|| r_squared_sum / r_squared_count as f64);

    match metric {
        ScoreMetric::RSquared => match mean_r_squared {
            Some(r2) => FoldOutcome {
                status: FoldStatus::Scored,
                r_squared: Some(r2),
                rmse: Some(mean_rmse),
                score: r2,
            },
            // No state had variance on this segment: R² is undefined, so the
            // fold cannot discriminate candidates and is recorded as a failure.
            None => FoldOutcome {
                status: FoldStatus::ScoringFailed,
                r_squared: None,
                rmse: Some(mean_rmse),
                score: FAILURE_SCORE,
            },
        },
        ScoreMetric::Rmse => FoldOutcome {
            status: FoldStatus::Scored,
            r_squared: mean_r_squared,
            rmse: Some(mean_rmse),
            score: -mean_rmse,
        },
    }
}

/// Whether `values` has any spread (a non-zero population variance signal).
fn has_variance(values: &[f64]) -> bool {
    let Some(&first) = values.first() else {
        return false;
    };
    values.iter().any(|value| *value != first)
}

/// Linearly interpolates `(source_times, source_values)` onto `query_times`.
///
/// Both series are strictly increasing; queries outside the source range clamp
/// to the nearest endpoint. Mirrors the CLI `validate` interpolation so scoring
/// is consistent with the shipped forecast diagnostics.
fn interpolate_onto(source_times: &[f64], source_values: &[f64], query_times: &[f64]) -> Vec<f64> {
    let mut cursor = 0;
    query_times
        .iter()
        .map(|&query| {
            while cursor + 1 < source_times.len() && source_times[cursor + 1] < query {
                cursor += 1;
            }
            if query <= source_times[0] {
                return source_values[0];
            }
            let last = source_times.len() - 1;
            if query >= source_times[last] {
                return source_values[last];
            }
            let left = cursor;
            let right = (cursor + 1).min(last);
            let span = source_times[right] - source_times[left];
            if span <= 0.0 {
                return source_values[left];
            }
            let fraction = (query - source_times[left]) / span;
            source_values[left] + fraction * (source_values[right] - source_values[left])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn interpolates_onto_observed_grid() {
        let source_times = vec![0.0, 1.0, 2.0];
        let source_values = vec![0.0, 10.0, 20.0];
        assert_eq!(interpolate_onto(&source_times, &source_values, &[0.5, 1.5]), vec![5.0, 15.0]);
    }

    #[test]
    fn detects_variance() {
        assert!(has_variance(&[1.0, 1.0, 2.0]));
        assert!(!has_variance(&[3.0, 3.0, 3.0]));
    }

    #[test]
    fn slices_preserve_columns_and_ordering() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0]).unwrap(),
            [NumericColumn::new(id("x"), vec![10.0, 11.0, 12.0, 13.0])],
        )
        .unwrap();
        let sliced = slice_dataset(&data, 1..3).unwrap();
        assert_eq!(sliced.time().values(), &[1.0, 2.0]);
        assert_eq!(sliced.columns()[&id("x")].values, vec![11.0, 12.0]);
    }
}
