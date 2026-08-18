//! Deterministic time-series fold planning.
//!
//! The timeline of `n` observations is cut into `folds + 1` contiguous segments
//! whose boundaries are pure integer functions of `(n, folds)` — no shuffling,
//! no floating point, so the plan is bit-reproducible. See [`plan_folds`].

use std::ops::Range;

use crate::{CvConfig, CvScheme, ModelSelectError};

/// One resolved fold: contiguous training and test index ranges into the
/// dataset's time axis. `test` immediately follows the data `train` was drawn
/// from, so scoring is always strictly forward in time.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldPlan {
    /// Zero-based fold position in evaluation order.
    pub index: usize,
    /// Training observation range `[start, end)`.
    pub train: Range<usize>,
    /// Held-out test observation range `[start, end)`, always after `train`.
    pub test: Range<usize>,
}

/// Segment boundary index `j` of `folds + 1` near-equal contiguous segments over
/// `n` samples: `floor(j * n / (folds + 1))`. Deterministic integer arithmetic.
fn boundary(j: usize, n: usize, segments: usize) -> usize {
    // `j * n` fits comfortably for realistic sizes; segments >= 1 always.
    (j * n) / segments
}

/// Resolves the fold plan for `n` samples under `cv`.
///
/// Returns [`ModelSelectError::InvalidFoldCount`] when no folds are requested and
/// [`ModelSelectError::DatasetTooShort`] when any fold would fall below the
/// configured train/test sample floors. The returned vector always has exactly
/// `cv.folds` entries in ascending time order.
pub fn plan_folds(n: usize, cv: &CvConfig) -> Result<Vec<FoldPlan>, ModelSelectError> {
    if cv.folds == 0 {
        return Err(ModelSelectError::InvalidFoldCount);
    }
    let segments = cv.folds + 1;
    let bounds: Vec<usize> = (0..=segments).map(|j| boundary(j, n, segments)).collect();

    let mut plans = Vec::with_capacity(cv.folds);
    for i in 0..cv.folds {
        let test = bounds[i + 1]..bounds[i + 2];
        let train = match cv.scheme {
            CvScheme::ForwardChaining => 0..bounds[i + 1],
            CvScheme::RollingBlocks => bounds[i]..bounds[i + 1],
        };
        if train.len() < cv.min_train_samples || test.len() < cv.min_test_samples {
            return Err(ModelSelectError::DatasetTooShort {
                samples: n,
                folds: cv.folds,
                min_train: cv.min_train_samples,
                min_test: cv.min_test_samples,
            });
        }
        plans.push(FoldPlan { index: i, train, test });
    }
    Ok(plans)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScoreMetric;

    fn cfg(scheme: CvScheme, folds: usize) -> CvConfig {
        CvConfig {
            scheme,
            folds,
            metric: ScoreMetric::RSquared,
            min_train_samples: 1,
            min_test_samples: 1,
        }
    }

    #[test]
    fn forward_chaining_grows_the_training_window() {
        let plans = plan_folds(100, &cfg(CvScheme::ForwardChaining, 3)).unwrap();
        assert_eq!(plans.len(), 3);
        // Four equal segments of 25: boundaries 0,25,50,75,100.
        assert_eq!(plans[0].train, 0..25);
        assert_eq!(plans[0].test, 25..50);
        assert_eq!(plans[1].train, 0..50);
        assert_eq!(plans[1].test, 50..75);
        assert_eq!(plans[2].train, 0..75);
        assert_eq!(plans[2].test, 75..100);
    }

    #[test]
    fn rolling_blocks_slide_a_fixed_window() {
        let plans = plan_folds(100, &cfg(CvScheme::RollingBlocks, 3)).unwrap();
        assert_eq!(plans[0].train, 0..25);
        assert_eq!(plans[1].train, 25..50);
        assert_eq!(plans[2].train, 50..75);
        assert_eq!(plans[2].test, 75..100);
    }

    #[test]
    fn rejects_zero_folds() {
        assert_eq!(
            plan_folds(100, &cfg(CvScheme::ForwardChaining, 0)),
            Err(ModelSelectError::InvalidFoldCount)
        );
    }

    #[test]
    fn rejects_datasets_too_short_for_the_floors() {
        let mut config = cfg(CvScheme::ForwardChaining, 5);
        config.min_train_samples = 3;
        config.min_test_samples = 2;
        // 6 samples / 6 segments = 1 per segment: first fold has train=1 < 3.
        assert!(matches!(
            plan_folds(6, &config),
            Err(ModelSelectError::DatasetTooShort { samples: 6, folds: 5, .. })
        ));
    }

    #[test]
    fn plan_is_deterministic() {
        let a = plan_folds(97, &cfg(CvScheme::ForwardChaining, 4)).unwrap();
        let b = plan_folds(97, &cfg(CvScheme::ForwardChaining, 4)).unwrap();
        assert_eq!(a, b);
    }
}
