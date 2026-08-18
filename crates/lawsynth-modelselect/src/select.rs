//! The cross-validated selection driver.
//!
//! For each candidate config the sweep runs the fold plan: discover on the
//! training segment, simulate the discovered world across the test segment, and
//! score predictive fit. The candidate's mean fold score decides selection, with
//! deterministic ties broken toward the simpler model.

use std::cmp::Ordering;

use lawsynth_data::Dataset;
use lawsynth_discovery::{DiscoveryConfig, discover};

use crate::score::{FoldOutcome, score_world_on_segment, slice_dataset};
use crate::{
    CandidateScore, ConfigSummary, CvConfig, FoldScore, FoldStatus, ModelSelectError,
    SelectionReport, plan_folds,
};

/// Runs a deterministic cross-validated hyperparameter sweep.
///
/// Evaluates every `DiscoveryConfig` in `grid` under the time-series CV scheme in
/// `cv`, returning a full [`SelectionReport`]: each candidate's per-fold and mean
/// held-out score plus the index of the best. Identical inputs yield a
/// bit-identical report.
///
/// # Errors
///
/// Returns [`ModelSelectError::EmptyGrid`] for an empty grid,
/// [`ModelSelectError::InvalidFoldCount`] for zero folds, and
/// [`ModelSelectError::DatasetTooShort`] when the dataset cannot be split into
/// the requested folds within the configured sample floors. A candidate that
/// fails discovery or simulation on a fold is *not* an error — it is recorded as
/// a per-fold failure.
pub fn select_model(
    dataset: &Dataset,
    grid: &[DiscoveryConfig],
    cv: &CvConfig,
) -> Result<SelectionReport, ModelSelectError> {
    if grid.is_empty() {
        return Err(ModelSelectError::EmptyGrid);
    }
    let folds = plan_folds(dataset.time().len(), cv)?;

    // Pre-slice the fold sub-datasets once; they are shared across every
    // candidate so the split is identical for all of them.
    let sliced: Vec<(Dataset, Dataset)> = folds
        .iter()
        .map(|fold| {
            let train = slice_dataset(dataset, fold.train.clone())?;
            let test = slice_dataset(dataset, fold.test.clone())?;
            Ok((train, test))
        })
        .collect::<Result<_, ModelSelectError>>()?;

    let candidates: Vec<CandidateScore> = grid
        .iter()
        .enumerate()
        .map(|(grid_index, config)| {
            evaluate_candidate(dataset, config, grid_index, cv, &folds, &sliced)
        })
        .collect();

    let best_index = select_best(&candidates);
    Ok(SelectionReport {
        candidates,
        best_index,
        scheme: cv.scheme,
        metric: cv.metric,
        folds: cv.folds,
    })
}

/// Convenience grid builder: sweeps the Cartesian product of candidate
/// `degrees` (outer) and `thresholds` (inner), cloning `base` for every cell and
/// overriding only the polynomial degree and sparsity threshold. All other knobs
/// (state variables, solver, derivative config, ...) come from `base`.
pub fn sweep_degrees_thresholds(
    dataset: &Dataset,
    base: &DiscoveryConfig,
    degrees: &[usize],
    thresholds: &[f64],
    cv: &CvConfig,
) -> Result<SelectionReport, ModelSelectError> {
    let mut grid = Vec::with_capacity(degrees.len() * thresholds.len());
    for &degree in degrees {
        for &threshold in thresholds {
            let mut config = base.clone();
            config.polynomial_degree = degree;
            config.sparse.threshold = threshold;
            grid.push(config);
        }
    }
    select_model(dataset, &grid, cv)
}

/// Scores one candidate over every fold and summarises it.
fn evaluate_candidate(
    dataset: &Dataset,
    config: &DiscoveryConfig,
    grid_index: usize,
    cv: &CvConfig,
    folds: &[crate::FoldPlan],
    sliced: &[(Dataset, Dataset)],
) -> CandidateScore {
    let mut fold_scores = Vec::with_capacity(folds.len());
    let mut score_sum = 0.0;
    let mut failed_folds = 0usize;
    for (fold, (train, test)) in folds.iter().zip(sliced) {
        let outcome = match discover(train, config) {
            Ok(result) => {
                let world = &result.candidates[0].world;
                score_world_on_segment(world, test, cv.metric)
            }
            Err(_) => FoldOutcome {
                status: FoldStatus::DiscoveryFailed,
                r_squared: None,
                rmse: None,
                score: crate::score::FAILURE_SCORE,
            },
        };
        if !outcome.status.is_scored() {
            failed_folds += 1;
        }
        score_sum += outcome.score;
        fold_scores.push(FoldScore {
            fold_index: fold.index,
            train_range: (fold.train.start, fold.train.end),
            test_range: (fold.test.start, fold.test.end),
            status: outcome.status,
            r_squared: outcome.r_squared,
            rmse: outcome.rmse,
            score: outcome.score,
        });
    }
    let mean_score = score_sum / folds.len() as f64;
    // Active-term count from a full-data refit: the model the winner will use,
    // and the "fewer active terms" tie-break signal. Never affects mean_score.
    let active_terms =
        discover(dataset, config).ok().map(|result| result.candidates[0].metrics.complexity);

    CandidateScore {
        config: ConfigSummary::from_config(config),
        grid_index,
        mean_score,
        fold_scores,
        failed_folds,
        active_terms,
    }
}

/// Picks the best candidate index: maximise mean score, then break ties toward
/// the simpler model. The documented tie-break order is:
///
/// 1. higher `mean_score`
/// 2. lower polynomial degree
/// 3. higher sparsity threshold
/// 4. fewer active terms (`None` sorts as the most complex)
/// 5. lower grid index
fn select_best(candidates: &[CandidateScore]) -> usize {
    let mut best = 0usize;
    for index in 1..candidates.len() {
        if is_better(&candidates[index], &candidates[best]) == Ordering::Greater {
            best = index;
        }
    }
    best
}

/// Total ordering where `Ordering::Greater` means `a` is the better (preferred)
/// candidate. Deterministic: every tier is a total order over finite data.
fn is_better(a: &CandidateScore, b: &CandidateScore) -> Ordering {
    a.mean_score
        .total_cmp(&b.mean_score)
        // Lower degree preferred: a wins when a.degree < b.degree.
        .then_with(|| b.config.polynomial_degree.cmp(&a.config.polynomial_degree))
        // Higher threshold preferred: a wins when a.threshold > b.threshold.
        .then_with(|| a.config.threshold.total_cmp(&b.config.threshold))
        // Fewer active terms preferred (None == usize::MAX, most complex).
        .then_with(|| terms_key(b).cmp(&terms_key(a)))
        // Lower grid index preferred: a wins when a.index < b.index.
        .then_with(|| b.grid_index.cmp(&a.grid_index))
}

fn terms_key(candidate: &CandidateScore) -> usize {
    candidate.active_terms.unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::report::ConfigSummary;
    use lawsynth_discovery::SparseMethod;

    fn summary(degree: usize, threshold: f64) -> ConfigSummary {
        ConfigSummary {
            polynomial_degree: degree,
            threshold,
            sparse_method: SparseMethod::Stlsq,
            include_trigonometric: false,
            include_rational: false,
        }
    }

    fn candidate(
        grid_index: usize,
        mean_score: f64,
        degree: usize,
        threshold: f64,
        active_terms: Option<usize>,
    ) -> CandidateScore {
        CandidateScore {
            config: summary(degree, threshold),
            grid_index,
            mean_score,
            fold_scores: Vec::new(),
            failed_folds: 0,
            active_terms,
        }
    }

    #[test]
    fn picks_the_highest_mean_score() {
        let candidates = vec![
            candidate(0, 0.5, 2, 0.05, Some(3)),
            candidate(1, 0.9, 3, 0.05, Some(5)),
            candidate(2, 0.7, 1, 0.05, Some(2)),
        ];
        assert_eq!(select_best(&candidates), 1);
    }

    #[test]
    fn breaks_ties_toward_lower_degree() {
        let candidates =
            vec![candidate(0, 0.9, 3, 0.05, Some(5)), candidate(1, 0.9, 2, 0.05, Some(5))];
        assert_eq!(select_best(&candidates), 1);
    }

    #[test]
    fn breaks_degree_ties_toward_higher_threshold_then_fewer_terms() {
        let candidates =
            vec![candidate(0, 0.9, 2, 0.05, Some(5)), candidate(1, 0.9, 2, 0.20, Some(5))];
        assert_eq!(select_best(&candidates), 1, "higher threshold preferred");

        let candidates =
            vec![candidate(0, 0.9, 2, 0.05, Some(5)), candidate(1, 0.9, 2, 0.05, Some(3))];
        assert_eq!(select_best(&candidates), 1, "fewer active terms preferred");
    }

    #[test]
    fn missing_active_terms_sort_as_most_complex() {
        let candidates =
            vec![candidate(0, 0.9, 2, 0.05, None), candidate(1, 0.9, 2, 0.05, Some(4))];
        assert_eq!(select_best(&candidates), 1);
    }
}
