//! The auditable selection report: every candidate's per-fold and mean scores.

use std::fmt::Write as _;

use lawsynth_discovery::{DiscoveryConfig, SparseMethod};

use crate::{CvScheme, ScoreMetric};

/// Outcome of scoring a single candidate on a single fold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldStatus {
    /// A predictive score was computed from a simulated held-out trajectory.
    Scored,
    /// Discovery failed on the training segment (e.g. resource limit, solver
    /// error); the fold is recorded as a failure, not silently dropped.
    DiscoveryFailed,
    /// The discovered world could not be simulated across the test segment
    /// (e.g. the trajectory diverged to a non-finite value).
    SimulationFailed,
    /// The trajectory simulated but could not be scored (e.g. the held-out
    /// window carried no variance to compute R² against).
    ScoringFailed,
}

impl FoldStatus {
    /// Whether a real predictive score was produced for this fold.
    pub fn is_scored(self) -> bool {
        matches!(self, FoldStatus::Scored)
    }

    fn label(self) -> &'static str {
        match self {
            FoldStatus::Scored => "ok",
            FoldStatus::DiscoveryFailed => "discover-fail",
            FoldStatus::SimulationFailed => "sim-fail",
            FoldStatus::ScoringFailed => "score-fail",
        }
    }
}

/// One candidate's result on one fold.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FoldScore {
    /// Zero-based fold index in evaluation order.
    pub fold_index: usize,
    /// Training range `[start, end)` used to discover the model.
    pub train_range: (usize, usize),
    /// Held-out test range `[start, end)` the model was scored on.
    pub test_range: (usize, usize),
    /// How the fold resolved.
    pub status: FoldStatus,
    /// Mean held-out R² over states, when computable.
    pub r_squared: Option<f64>,
    /// Mean held-out RMSE over states, when computable.
    pub rmse: Option<f64>,
    /// Higher-is-better selection score for this fold. On failure this is the
    /// documented worst-case [`crate::score::FAILURE_SCORE`].
    pub score: f64,
}

/// A lightweight, comparable summary of the swept hyperparameters, sufficient to
/// audit which knobs a candidate used without cloning the whole config.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ConfigSummary {
    /// Candidate-library polynomial degree.
    pub polynomial_degree: usize,
    /// Sparsity threshold applied to fitted coefficients.
    pub threshold: f64,
    /// Sparse solver used for the coefficient fit.
    pub sparse_method: SparseMethod,
    /// Whether trigonometric library terms were included.
    pub include_trigonometric: bool,
    /// Whether bounded-rational library terms were included.
    pub include_rational: bool,
}

impl ConfigSummary {
    /// Extracts the swept-knob summary from a full discovery config.
    pub fn from_config(config: &DiscoveryConfig) -> Self {
        Self {
            polynomial_degree: config.polynomial_degree,
            threshold: config.sparse.threshold,
            sparse_method: config.sparse_method,
            include_trigonometric: config.include_trigonometric,
            include_rational: config.include_rational,
        }
    }
}

/// A single candidate's full cross-validated score.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateScore {
    /// Summary of the hyperparameters this candidate used.
    pub config: ConfigSummary,
    /// Index of this candidate in the user-supplied grid.
    pub grid_index: usize,
    /// Mean of the per-fold [`FoldScore::score`] values (including failures).
    pub mean_score: f64,
    /// Per-fold results, one per fold in evaluation order.
    pub fold_scores: Vec<FoldScore>,
    /// Number of folds that did not produce a real predictive score.
    pub failed_folds: usize,
    /// Complexity (active-term count) of the model discovered on the *full*
    /// dataset with this config, used for the simpler-model tie-break and as an
    /// audit signal. `None` when full-data discovery failed.
    pub active_terms: Option<usize>,
}

/// The full auditable outcome of a selection sweep.
#[derive(Clone, Debug, PartialEq)]
pub struct SelectionReport {
    /// Every candidate's score, in grid order.
    pub candidates: Vec<CandidateScore>,
    /// Index into [`candidates`](Self::candidates) of the selected best model.
    pub best_index: usize,
    /// Fold-assignment scheme used.
    pub scheme: CvScheme,
    /// Metric that drove selection.
    pub metric: ScoreMetric,
    /// Number of folds evaluated per candidate.
    pub folds: usize,
}

impl SelectionReport {
    /// The selected best candidate.
    pub fn best(&self) -> &CandidateScore {
        &self.candidates[self.best_index]
    }

    /// Renders the full candidate score table as human-readable text, so the
    /// selection is auditable at a glance (every candidate, not just the winner).
    pub fn render_table(&self) -> String {
        let mut out = String::new();
        let metric = match self.metric {
            ScoreMetric::RSquared => "R^2",
            ScoreMetric::Rmse => "-RMSE",
        };
        let scheme = match self.scheme {
            CvScheme::ForwardChaining => "forward-chaining",
            CvScheme::RollingBlocks => "rolling-blocks",
        };
        let _ = writeln!(
            out,
            "Model selection: {} folds, {scheme} CV, metric={metric} (higher is better)",
            self.folds
        );
        let _ = writeln!(
            out,
            "  {:>4} {:>6} {:>10} {:>6} {:>14} {:>6} {:>7}",
            "idx", "degree", "threshold", "terms", "mean_score", "fails", "best"
        );
        for (index, candidate) in self.candidates.iter().enumerate() {
            let _ = writeln!(
                out,
                "  {:>4} {:>6} {:>10} {:>6} {:>14} {:>6} {:>7}",
                candidate.grid_index,
                candidate.config.polynomial_degree,
                format!("{:.4}", candidate.config.threshold),
                candidate
                    .active_terms
                    .map(|terms| terms.to_string())
                    .unwrap_or_else(|| "n/a".to_owned()),
                format!("{:.6e}", candidate.mean_score),
                candidate.failed_folds,
                if index == self.best_index { "<==" } else { "" },
            );
        }
        for candidate in &self.candidates {
            let _ = write!(out, "  grid[{}] folds:", candidate.grid_index);
            for fold in &candidate.fold_scores {
                let _ = write!(
                    out,
                    " [{}:{}]",
                    fold.status.label(),
                    match fold.r_squared {
                        Some(r2) => format!("R2={r2:.4}"),
                        None => format!("score={:.3e}", fold.score),
                    }
                );
            }
            out.push('\n');
        }
        out
    }
}
