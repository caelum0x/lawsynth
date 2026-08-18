//! Configuration for the deterministic time-series cross-validation sweep.

/// How contiguous time segments are assigned to training and test folds.
///
/// Both schemes partition the timeline into `folds + 1` contiguous, near-equal
/// segments and never shuffle rows, so temporal order is preserved. Fold `i`
/// (for `i` in `0..folds`) always **tests on segment `i + 1`**; the schemes
/// differ only in which earlier data they train on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CvScheme {
    /// Expanding-window forward chaining (the standard time-series CV): fold `i`
    /// trains on *all* observations up to the start of test segment `i + 1`
    /// (segments `0..=i`) and tests on segment `i + 1`. The training window grows
    /// each fold; the model is always fit on the past and scored on the future.
    ForwardChaining,
    /// Rolling fixed-block window: fold `i` trains on segment `i` only and tests
    /// on the immediately following segment `i + 1`. The training window is one
    /// block wide and slides forward, so every fold trains on a comparable amount
    /// of the most recent past.
    RollingBlocks,
}

/// Which predictive metric drives selection. Both are computed per fold; the
/// selected one becomes the fold's higher-is-better `score`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScoreMetric {
    /// Coefficient of determination of the re-simulated trajectory against the
    /// held-out observations, averaged over states. Higher is better; the fold
    /// score is the mean R² directly.
    RSquared,
    /// Root-mean-squared error of the re-simulated trajectory against the
    /// held-out observations, averaged over states. Lower is better; the fold
    /// score is the *negated* mean RMSE so that, like R², higher is better.
    Rmse,
}

/// Deterministic time-series cross-validation settings.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CvConfig {
    /// Fold assignment scheme (no random shuffling).
    pub scheme: CvScheme,
    /// Number of train/test folds. The timeline is cut into `folds + 1`
    /// contiguous segments; each fold tests on one segment.
    pub folds: usize,
    /// Metric that decides the best candidate.
    pub metric: ScoreMetric,
    /// Minimum training observations a fold must have; discovery needs at least
    /// three samples, so this is the floor for the default.
    pub min_train_samples: usize,
    /// Minimum test observations a fold must have; scoring needs at least two so
    /// the held-out window spans a non-empty time interval.
    pub min_test_samples: usize,
}

impl CvConfig {
    /// Builds a forward-chaining, R²-scored configuration with the given fold
    /// count and conservative sample floors.
    pub fn new(folds: usize) -> Self {
        Self {
            scheme: CvScheme::ForwardChaining,
            folds,
            metric: ScoreMetric::RSquared,
            min_train_samples: 3,
            min_test_samples: 2,
        }
    }

    /// Selects the fold-assignment scheme.
    pub fn with_scheme(mut self, scheme: CvScheme) -> Self {
        self.scheme = scheme;
        self
    }

    /// Selects the predictive metric that drives selection.
    pub fn with_metric(mut self, metric: ScoreMetric) -> Self {
        self.metric = metric;
        self
    }

    /// Overrides the minimum per-fold training and test sample floors.
    pub fn with_sample_floors(mut self, min_train: usize, min_test: usize) -> Self {
        self.min_train_samples = min_train;
        self.min_test_samples = min_test;
        self
    }
}
