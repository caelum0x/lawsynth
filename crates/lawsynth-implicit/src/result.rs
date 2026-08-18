use lawsynth_differentiate::DerivativeMethod;

use crate::AugmentedTerm;
use crate::rational::RationalLaw;

/// One coefficient of a discovered implicit relation `Θ(x, ẋ) ξ ≈ 0`.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitTerm {
    /// The augmented library column this coefficient belongs to.
    pub term: AugmentedTerm,
    /// The (normalised) coefficient `ξⱼ`. The chosen left-hand-side term is
    /// fixed to exactly `1.0`.
    pub coefficient: f64,
}

/// A sparse non-trivial relation `f(x, ẋ) = Θ(x, ẋ) ξ ≈ 0`.
///
/// The relation is normalised so that the selected left-hand-side term has
/// coefficient `1`, which is how the trivial `ξ = 0` solution is excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitRelation {
    /// All non-zero coefficients of `ξ`, in library order.
    pub terms: Vec<ImplicitTerm>,
    /// Index (into the full augmented library) of the normalised term.
    pub lhs_index: usize,
    /// Human-readable name of the normalised left-hand-side term.
    pub lhs_name: String,
    /// Residual sum of squares of the alternating-LHS fit that produced this
    /// relation, in the original (un-standardised) units.
    pub residual: f64,
    /// `residual / Σ (Θ_lhs)²` — a scale-free measure of consistency.
    pub relative_residual: f64,
    /// Number of non-zero coefficients (including the normalised term).
    pub active_terms: usize,
    /// Whether `relative_residual` is within the configured tolerance.
    pub consistent: bool,
}

/// The score assigned to one candidate left-hand-side column.
#[derive(Clone, Debug, PartialEq)]
pub struct CandidateScore {
    pub lhs_index: usize,
    pub lhs_name: String,
    pub relative_residual: f64,
    pub active_terms: usize,
    /// Combined objective `relative_residual + sparsity_weight · active/library`.
    pub score: f64,
    /// `false` when the column was degenerate (identically zero, or a singular
    /// fit) and therefore skipped during selection.
    pub usable: bool,
}

/// Reproducibility and honesty diagnostics for an implicit discovery run.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitDiagnostics {
    pub target: String,
    pub samples: usize,
    pub library_size: usize,
    pub candidates_evaluated: usize,
    pub usable_candidates: usize,
    pub derivative_method: DerivativeMethod,
    pub best_relative_residual: f64,
    pub dataset_fingerprint: u64,
    /// Per-candidate scores in library order (for auditability).
    pub candidate_scores: Vec<CandidateScore>,
}

/// The full result of implicit / rational dynamics discovery.
#[derive(Clone, Debug, PartialEq)]
pub struct ImplicitResult {
    /// The discovered implicit relation `Θ(x, ẋ) ξ ≈ 0` (always present).
    pub relation: ImplicitRelation,
    /// The explicit rational law `ẋ = P(x) / Q(x)`, reconstructed when the
    /// relation is consistent and genuinely involves the derivative.
    pub rational_law: Option<RationalLaw>,
    pub diagnostics: ImplicitDiagnostics,
}
