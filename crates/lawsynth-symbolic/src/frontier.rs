use lawsynth_expr::Expr;

/// A scored expression with a minimization loss and structural complexity.
#[derive(Clone, Debug, PartialEq)]
pub struct ScoredExpression {
    pub expression: Expr,
    pub loss: f64,
    pub complexity: usize,
}

/// Returns non-dominated symbolic candidates in input order.
pub fn pareto_by_loss_and_complexity(candidates: &[ScoredExpression]) -> Vec<usize> {
    candidates
        .iter()
        .enumerate()
        .filter_map(|(index, candidate)| {
            (!candidates.iter().enumerate().any(|(other_index, other)| {
                other_index != index
                    && (other.loss <= candidate.loss && other.complexity <= candidate.complexity)
                    && (other.loss < candidate.loss || other.complexity < candidate.complexity)
            }))
            .then_some(index)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn excludes_dominated_candidate() {
        let values = [
            ScoredExpression { expression: Expr::constant(1.0), loss: 1.0, complexity: 1 },
            ScoredExpression { expression: Expr::constant(2.0), loss: 2.0, complexity: 2 },
        ];
        assert_eq!(pareto_by_loss_and_complexity(&values), vec![0]);
    }
}
