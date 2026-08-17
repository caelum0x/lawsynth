use lawsynth_expr::{Environment, Expr, evaluate};
use lawsynth_opt::AffineFit;

use crate::SymbolicError;

/// A symbolic expression with the best affine constants for supplied data.
#[derive(Clone, Debug, PartialEq)]
pub struct CalibratedExpression {
    pub expression: Expr,
    pub fit: AffineFit,
}

/// Fits `target ~= scale * expression(context) + offset` deterministically.
///
/// Affine calibration turns grammar-only structural candidates into executable
/// laws while retaining the base expression for transparent ranking.
pub fn calibrate_affine(
    expression: &Expr,
    contexts: &[Environment],
    targets: &[f64],
) -> Result<CalibratedExpression, SymbolicError> {
    if contexts.is_empty() {
        return Err(SymbolicError::EmptyInput);
    }
    if contexts.len() != targets.len() {
        return Err(SymbolicError::LengthMismatch);
    }
    let predictions = contexts
        .iter()
        .map(|context| {
            evaluate(expression, context)
                .map_err(|error| SymbolicError::Evaluation(error.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let fit = lawsynth_opt::fit_affine(&predictions, targets)
        .map_err(|error| SymbolicError::Optimization(error.to_string()))?;
    let calibrated = Expr::sum(
        Expr::product(Expr::constant(fit.scale), expression.clone()),
        Expr::constant(fit.offset),
    )
    .simplify();
    Ok(CalibratedExpression { expression: calibrated, fit })
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    #[test]
    fn calibrates_a_grammar_candidate() {
        let x = Identifier::new("x").unwrap();
        let expression = Expr::symbol(x.clone());
        let contexts = [1.0, 2.0, 3.0]
            .into_iter()
            .map(|value| Environment::from([(x.clone(), value)]))
            .collect::<Vec<_>>();
        let candidate = calibrate_affine(&expression, &contexts, &[5.0, 7.0, 9.0]).unwrap();
        assert_eq!(candidate.fit.scale, 2.0);
        assert_eq!(candidate.fit.offset, 3.0);
        assert_eq!(evaluate(&candidate.expression, &contexts[1]).unwrap(), 7.0);
    }
}
