use lawsynth_core::Identifier;
use lawsynth_sparse::SparseConfig;

use crate::SdeError;

/// How the observed state space is partitioned into bins for the conditional
/// moment (Kramers–Moyal) estimates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BinRule {
    /// A fixed number of equal-width bins spanning `[min, max]` of the state.
    Count(usize),
    /// Equal-width bins of the given width, starting at the state minimum.
    Width(f64),
}

/// Configuration for [`crate::discover_sde`].
///
/// All fields are explicit and deterministic — there is no hidden randomness and
/// no wall-clock input. Identical `(Dataset, SdeConfig)` pairs always produce a
/// bit-identical [`crate::SdeModel`].
#[derive(Clone, Debug, PartialEq)]
pub struct SdeConfig {
    /// The number of equal-length trajectories concatenated in the dataset. The
    /// rows are split into this many contiguous segments and increments are only
    /// ever formed *within* a segment, never across a boundary — so an ensemble
    /// of independent sample paths can be passed as one dataset without spurious
    /// jump increments. `1` (the default) treats the dataset as a single path.
    pub trajectories: usize,
    /// State columns to estimate drift/diffusion for. Empty means *every* numeric
    /// column in the dataset, processed in the dataset's (lexicographic) schema
    /// order. Each state is binned by its *own* value — this recovers the drift
    /// and diffusion of a diagonal-noise Itô SDE `dXⱼ = aⱼ(Xⱼ) dt + bⱼ(Xⱼ) dWⱼ`.
    pub state_columns: Vec<Identifier>,
    /// The state-space partition rule.
    pub bin_rule: BinRule,
    /// The minimum number of increments a bin must contain before it is trusted
    /// and fed to the sparse regression. Rarely-visited bins are noisy.
    pub min_bin_count: usize,
    /// Degree of the polynomial candidate library the binned drift and diffusion
    /// are regressed onto.
    pub polynomial_degree: usize,
    /// Whether the library includes a constant term (the intercept, e.g. a
    /// constant diffusion `σ²`).
    pub include_constant: bool,
    /// If set, [`crate::discover_sde`] rejects an irregularly spaced time axis
    /// instead of silently proceeding. The Kramers–Moyal expansion assumes a
    /// small, consistent `Δt`.
    pub require_regular_time: bool,
    /// Relative tolerance for the regular-spacing check (see
    /// [`lawsynth_data::TimeAxis::is_regular`]).
    pub time_regular_tolerance: f64,
    /// If set, the sparse fit weights each bin by its occupancy (a weighted
    /// least squares with weight `count`). The variance of a bin's mean estimate
    /// scales like `1/count`, so this down-weights sparsely-visited (noisy) bins
    /// and is the statistically correct default.
    pub weight_by_occupancy: bool,
    /// Sparse regression settings used to fit the binned estimates.
    pub sparse: SparseConfig,
}

impl Default for SdeConfig {
    fn default() -> Self {
        Self {
            trajectories: 1,
            state_columns: Vec::new(),
            bin_rule: BinRule::Count(24),
            min_bin_count: 30,
            polynomial_degree: 3,
            include_constant: true,
            require_regular_time: true,
            time_regular_tolerance: 1e-6,
            weight_by_occupancy: true,
            sparse: SparseConfig { threshold: 0.05, max_iterations: 20, ridge: 1e-10 },
        }
    }
}

impl SdeConfig {
    /// A default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Restricts discovery to the given state columns.
    pub fn with_state_columns(mut self, columns: impl IntoIterator<Item = Identifier>) -> Self {
        self.state_columns = columns.into_iter().collect();
        self
    }

    /// Sets the state-space partition rule.
    pub fn with_bins(mut self, rule: BinRule) -> Self {
        self.bin_rule = rule;
        self
    }

    /// Sets the minimum trusted bin occupancy.
    pub fn with_min_bin_count(mut self, count: usize) -> Self {
        self.min_bin_count = count;
        self
    }

    /// Sets the candidate library polynomial degree.
    pub fn with_polynomial_degree(mut self, degree: usize) -> Self {
        self.polynomial_degree = degree;
        self
    }

    /// Overrides the sparse regression settings.
    pub fn with_sparse(mut self, sparse: SparseConfig) -> Self {
        self.sparse = sparse;
        self
    }

    /// Declares that the dataset concatenates `count` equal-length trajectories.
    pub fn with_trajectories(mut self, count: usize) -> Self {
        self.trajectories = count;
        self
    }

    /// The number of candidate library terms for the configured degree.
    pub(crate) fn library_term_count(&self) -> usize {
        if self.include_constant { self.polynomial_degree + 1 } else { self.polynomial_degree }
    }

    /// Validates the numeric fields independent of any dataset.
    pub(crate) fn validate(&self) -> Result<(), SdeError> {
        if self.polynomial_degree == 0 {
            return Err(SdeError::InvalidConfig("polynomial_degree must be >= 1".to_owned()));
        }
        if self.min_bin_count == 0 {
            return Err(SdeError::InvalidConfig("min_bin_count must be >= 1".to_owned()));
        }
        if self.trajectories == 0 {
            return Err(SdeError::InvalidConfig("trajectories must be >= 1".to_owned()));
        }
        if !self.time_regular_tolerance.is_finite() || self.time_regular_tolerance < 0.0 {
            return Err(SdeError::InvalidConfig(
                "time_regular_tolerance must be finite and non-negative".to_owned(),
            ));
        }
        if self.library_term_count() == 0 {
            return Err(SdeError::InvalidConfig(
                "candidate library would be empty; enable the constant term or raise the degree"
                    .to_owned(),
            ));
        }
        match self.bin_rule {
            BinRule::Count(count) => {
                if count == 0 {
                    return Err(SdeError::InvalidConfig("bin count must be >= 1".to_owned()));
                }
            }
            BinRule::Width(width) => {
                if !width.is_finite() || width <= 0.0 {
                    return Err(SdeError::InvalidConfig(
                        "bin width must be finite and positive".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }
}
