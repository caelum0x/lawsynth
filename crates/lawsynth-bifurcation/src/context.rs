//! A small bundle that re-locates fixed points at an arbitrary parameter value.
//!
//! Continuation and bifurcation localization repeatedly ask "what are the fixed
//! points at `μ`?". This context captures the field, states, parameter, and
//! stability configuration once, and answers that question by substituting `μ`
//! into the field and delegating to [`lawsynth_stability::analyze_stability`].

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_stability::{StabilityConfig, StabilityReport, analyze_stability};

use crate::error::BifurcationError;
use crate::substitute::substitute;

/// Borrows the inputs needed to evaluate stability at any parameter value.
pub(crate) struct FieldContext<'a> {
    fields: &'a [(Identifier, Expr)],
    states: &'a [Identifier],
    parameter: &'a Identifier,
    stability: &'a StabilityConfig,
}

impl<'a> FieldContext<'a> {
    /// Builds a context over borrowed inputs.
    pub(crate) fn new(
        fields: &'a [(Identifier, Expr)],
        states: &'a [Identifier],
        parameter: &'a Identifier,
        stability: &'a StabilityConfig,
    ) -> Self {
        Self { fields, states, parameter, stability }
    }

    /// Locates and classifies the fixed points of the field at `parameter_value`.
    ///
    /// The parameter symbol is substituted out first, yielding an autonomous
    /// field, so a residual stability fault is reported against this exact value.
    pub(crate) fn at(&self, parameter_value: f64) -> Result<StabilityReport, BifurcationError> {
        let bound: Vec<(Identifier, Expr)> = self
            .fields
            .iter()
            .map(|(target, expression)| {
                (target.clone(), substitute(expression, self.parameter, parameter_value))
            })
            .collect();
        analyze_stability(&bound, self.states, self.stability)
            .map_err(|source| BifurcationError::Stability { parameter_value, source })
    }
}
