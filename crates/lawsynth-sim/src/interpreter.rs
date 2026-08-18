use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_expr::evaluate;

use crate::{CompiledContinuousWorld, CompiledDiscreteWorld, SimulationContext, SimulationError};

/// Evaluates a continuous law plan against one immutable runtime context.
pub fn evaluate_continuous(
    compiled: &CompiledContinuousWorld,
    context: &SimulationContext,
) -> Result<BTreeMap<Identifier, f64>, SimulationError> {
    evaluate_plan(compiled.laws(), context)
}

/// Evaluates a simultaneous discrete update plan against one runtime context.
pub fn evaluate_discrete(
    compiled: &CompiledDiscreteWorld,
    context: &SimulationContext,
) -> Result<BTreeMap<Identifier, f64>, SimulationError> {
    evaluate_plan(compiled.laws(), context)
}

fn evaluate_plan(
    laws: &[(Identifier, lawsynth_expr::Expr)],
    context: &SimulationContext,
) -> Result<BTreeMap<Identifier, f64>, SimulationError> {
    let environment = context.environment();
    laws.iter()
        .map(|(id, expression)| {
            let value = evaluate(expression, &environment)?;
            if !value.is_finite() {
                return Err(SimulationError::NonFiniteInput { name: id.clone(), value });
            }
            Ok((id.clone(), value))
        })
        .collect()
}
