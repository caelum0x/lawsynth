use lawsynth_sparse::{RegressionProblem, SparseConfig, SparseError, stlsq_standardized};

use crate::ImplicitError;
use crate::config::ImplicitConfig;
use crate::library::AugmentedMatrix;
use crate::result::{CandidateScore, ImplicitRelation, ImplicitTerm};

const ZERO_TARGET: f64 = 1e-12;
const ACTIVE: f64 = 1e-9;
/// Relative residuals below this are all "perfect" fits — distinguishing them by
/// machine-noise-level differences (~1e-15) would let numerical noise, rather than
/// sparsity or canonical form, decide which implicit relation is chosen. Floored to
/// zero in the SELECTION score (the reported diagnostic residual keeps its true value).
const RESIDUAL_FLOOR: f64 = 1e-8;

/// Solves the implicit relation via the SINDy-PI alternating-LHS scheme.
///
/// For each augmented column `j` we move it to the left-hand side and fit the
/// sparse regression `Θ_j ≈ Θ_{\j} c`. The residual of this fit measures how
/// consistently the relation `Θ_j - Θ_{\j} c = 0` holds; the number of retained
/// terms measures its sparsity. We score every candidate and keep the best,
/// which fixes the trivial `ξ = 0` solution because the chosen column's
/// coefficient is normalised to exactly `1`.
pub(crate) fn solve_implicit(
    matrix: &AugmentedMatrix,
    config: &ImplicitConfig,
) -> Result<(ImplicitRelation, Vec<CandidateScore>), ImplicitError> {
    let width = matrix.terms.len();
    if width < 2 {
        return Err(ImplicitError::NoRelation);
    }
    let sparse_config = SparseConfig {
        threshold: config.threshold,
        max_iterations: config.max_iterations,
        ridge: config.ridge,
    };

    let mut scores = Vec::with_capacity(width);
    let mut best: Option<Candidate> = None;
    for lhs in 0..width {
        let outcome = fit_candidate(matrix, lhs, &sparse_config, config, width);
        scores.push(outcome.score.clone());
        if let Some(candidate) = outcome.candidate {
            let replace = match &best {
                None => true,
                Some(current) => is_better(&candidate, current),
            };
            if replace {
                best = Some(candidate);
            }
        }
    }

    let best = best.ok_or(ImplicitError::NoRelation)?;
    let relation = build_relation(matrix, &best, config);
    Ok((relation, scores))
}

struct Candidate {
    lhs: usize,
    coefficients: Vec<f64>,
    residual: f64,
    relative_residual: f64,
    active_terms: usize,
    score: f64,
    /// Whether the chosen left-hand side term carries the target derivative `ẋ`.
    /// Preferring a derivative-bearing LHS resolves the multi-dimensional-nullspace
    /// tie toward the canonical relation `Q(x)·ẋ = P(x)`, which reconstructs to the
    /// lowest-degree explicit rational law rather than an equally-valid but
    /// non-canonical pure-state relation.
    lhs_involves_derivative: bool,
}

struct Outcome {
    score: CandidateScore,
    candidate: Option<Candidate>,
}

fn fit_candidate(
    matrix: &AugmentedMatrix,
    lhs: usize,
    sparse_config: &SparseConfig,
    config: &ImplicitConfig,
    width: usize,
) -> Outcome {
    let name = matrix.terms[lhs].name.clone();
    let target = matrix.rows.iter().map(|row| row[lhs]).collect::<Vec<_>>();
    let target_energy = target.iter().map(|value| value * value).sum::<f64>();

    // A left-hand side that is identically zero cannot be normalised to `1`;
    // skipping it is exactly how the trivial nullspace vector is excluded.
    if target_energy < ZERO_TARGET {
        return Outcome {
            score: CandidateScore {
                lhs_index: lhs,
                lhs_name: name,
                relative_residual: f64::INFINITY,
                active_terms: 0,
                score: f64::INFINITY,
                usable: false,
            },
            candidate: None,
        };
    }

    let feature_rows = matrix
        .rows
        .iter()
        .map(|row| {
            row.iter()
                .enumerate()
                .filter(|(index, _)| *index != lhs)
                .map(|(_, value)| *value)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let solution = RegressionProblem::new(feature_rows, target.clone())
        .and_then(|problem| stlsq_standardized(&problem, sparse_config));
    let solution = match solution {
        Ok(solution) => solution,
        Err(SparseError::SingularSystem) | Err(SparseError::EmptyProblem) => {
            return unusable(lhs, name);
        }
        Err(_) => return unusable(lhs, name),
    };

    let relative_residual = solution.residual_sum_squares / target_energy;
    // Map reduced-feature coefficients back to full library positions.
    let mut coefficients = vec![0.0; width];
    coefficients[lhs] = 1.0;
    let mut feature_index = 0;
    for (column, coefficient) in coefficients.iter_mut().enumerate() {
        if column == lhs {
            continue;
        }
        *coefficient = -solution.coefficients[feature_index];
        feature_index += 1;
    }
    let active_terms = coefficients.iter().filter(|c| c.abs() >= ACTIVE).count();
    // Near-perfect fits are equivalent; floor sub-tolerance residuals so the
    // sparsity term and the derivative/index tie-break — not machine noise —
    // pick among them.
    let effective_residual =
        if relative_residual < RESIDUAL_FLOOR { 0.0 } else { relative_residual };
    let score = effective_residual + config.sparsity_weight * (active_terms as f64 / width as f64);

    Outcome {
        score: CandidateScore {
            lhs_index: lhs,
            lhs_name: name,
            relative_residual,
            active_terms,
            score,
            usable: true,
        },
        candidate: Some(Candidate {
            lhs,
            coefficients,
            residual: solution.residual_sum_squares,
            relative_residual,
            active_terms,
            score,
            lhs_involves_derivative: matrix.terms[lhs].involves_derivative,
        }),
    }
}

fn unusable(lhs: usize, name: String) -> Outcome {
    Outcome {
        score: CandidateScore {
            lhs_index: lhs,
            lhs_name: name,
            relative_residual: f64::INFINITY,
            active_terms: 0,
            score: f64::INFINITY,
            usable: false,
        },
        candidate: None,
    }
}

/// Deterministic ordering: lower score wins; ties break toward fewer active
/// terms, then toward a derivative-bearing LHS (the canonical `Q(x)·ẋ = P(x)`
/// form), then toward the earlier library index. The derivative preference is
/// what resolves the implicit multi-dimensional-nullspace ambiguity at higher
/// degree in favour of the lowest-degree explicit rational law.
fn is_better(candidate: &Candidate, current: &Candidate) -> bool {
    match candidate.score.total_cmp(&current.score) {
        std::cmp::Ordering::Less => true,
        std::cmp::Ordering::Greater => false,
        std::cmp::Ordering::Equal => match candidate.active_terms.cmp(&current.active_terms) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => {
                match (candidate.lhs_involves_derivative, current.lhs_involves_derivative) {
                    (true, false) => true,
                    (false, true) => false,
                    _ => candidate.lhs < current.lhs,
                }
            }
        },
    }
}

fn build_relation(
    matrix: &AugmentedMatrix,
    candidate: &Candidate,
    config: &ImplicitConfig,
) -> ImplicitRelation {
    let terms = matrix
        .terms
        .iter()
        .zip(&candidate.coefficients)
        .filter(|(_, coefficient)| coefficient.abs() >= ACTIVE)
        .map(|(term, coefficient)| ImplicitTerm { term: term.clone(), coefficient: *coefficient })
        .collect::<Vec<_>>();
    ImplicitRelation {
        terms,
        lhs_index: candidate.lhs,
        lhs_name: matrix.terms[candidate.lhs].name.clone(),
        residual: candidate.residual,
        relative_residual: candidate.relative_residual,
        active_terms: candidate.active_terms,
        consistent: candidate.relative_residual <= config.consistency_tolerance,
    }
}
