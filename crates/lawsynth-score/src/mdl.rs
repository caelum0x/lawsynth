//! Minimum-description-length (MDL) scoring for discovered models.
//!
//! MDL is the "two-part code" objective used by AI-Feynman (as its selection
//! currency) and Operon's `MinimumDescriptionLengthEvaluator`: a good model is
//! the one that lets you transmit the data in the fewest total nats. The score
//! is the sum of two code lengths,
//!
//! ```text
//! DL(model, data) = L(data | model) + L(model)
//! ```
//!
//! where the **data term** `L(data | model)` is the Gaussian negative
//! log-likelihood of the residuals at the maximum-likelihood variance, and the
//! **model term** `L(model)` charges for the structure (how many nodes, drawn
//! from how large an operator/terminal alphabet) plus the numeric constants and
//! the precision needed to pin them down.
//!
//! Every quantity is a deterministic function of its inputs (no clocks, no RNG,
//! no floating-point reductions whose order changes between runs), so the same
//! model and data always produce identical bits.

use crate::ScoreError;
use crate::complexity::expression_complexity;
use lawsynth_expr::{Expr, symbols};
use std::f64::consts::TAU;

/// Number of distinct operators the scalar expression grammar can emit
/// (`Negate, Exp, Log, Sin, Cos` unary + `Add, Subtract, Multiply, Divide,
/// Power` binary). Used as the structural part of the coding alphabet.
pub const OPERATOR_VOCABULARY: usize = 10;

/// Structural summary of a candidate model, i.e. everything the model code
/// length is charged for.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelDescription {
    /// Total AST node count (operators + terminals).
    pub nodes: usize,
    /// Number of operator (internal) nodes.
    pub operators: usize,
    /// Fitted constant leaf values, in AST traversal order.
    pub constants: Vec<f64>,
    /// Size of the coding alphabet: distinct operators + distinct variable
    /// symbols + one token for "a constant follows". Must be at least one so
    /// the per-node structural cost `ln(alphabet)` is non-negative.
    pub alphabet: usize,
}

impl ModelDescription {
    /// Derives a description from an expression: counts nodes, operator nodes,
    /// and constant leaves, and sizes the alphabet from the grammar's operator
    /// vocabulary plus the distinct symbols actually referenced.
    pub fn from_expression(expression: &Expr) -> Self {
        let mut operators = 0usize;
        let mut constants = Vec::new();
        collect(expression, &mut operators, &mut constants);
        let distinct_symbols = symbols(expression).len();
        ModelDescription {
            nodes: expression_complexity(expression),
            operators,
            constants,
            // +1 reserves a codeword for the "constant" terminal class.
            alphabet: OPERATOR_VOCABULARY + distinct_symbols + 1,
        }
    }
}

fn collect(expression: &Expr, operators: &mut usize, constants: &mut Vec<f64>) {
    match expression {
        Expr::Constant(value) => constants.push(*value),
        Expr::Symbol(_) => {}
        Expr::Unary { operand, .. } => {
            *operators += 1;
            collect(operand, operators, constants);
        }
        Expr::Binary { left, right, .. } => {
            *operators += 1;
            collect(left, operators, constants);
            collect(right, operators, constants);
        }
    }
}

/// The two-part MDL code length of a model fitted to `observations` points,
/// all quantities expressed in **nats** (natural-log units).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DescriptionLength {
    /// `L(data | model) + L(model)`; the value to minimize.
    pub total: f64,
    /// Gaussian negative log-likelihood of the residuals at the MLE variance.
    pub data_code_length: f64,
    /// Structural cost plus constant-encoding cost of the model.
    pub model_code_length: f64,
}

/// Computes the minimum-description-length score of a model.
///
/// # Formula (nats)
///
/// Let `n = observations`, `RSS = residual_sum_squares`, and
/// `sigma2 = max(RSS / n, MIN_POSITIVE)` (the MLE residual variance, clamped so
/// a perfect fit stays finite, exactly as [`crate::information_criteria`] does).
///
/// * **Data term** — Gaussian NLL at the MLE:
///   `L_data = (n / 2) * ln(2*pi*sigma2) + RSS / (2*sigma2)`.
///   This is strictly increasing in `RSS` (better fit ⇒ smaller code length).
/// * **Model term** — structure + constants:
///   `L_model = nodes * ln(alphabet) + sum_c [ 0.5 * ln(n) + ln(1 + |c|) ]`.
///   The `nodes * ln(alphabet)` term charges `ln(alphabet)` nats to name each
///   node (strictly increasing in node count). Each constant costs Rissanen's
///   `0.5 * ln(n)` precision term (the same per-parameter penalty BIC uses)
///   plus `ln(1 + |c|)` to encode its magnitude.
///
/// # Monotonicity
///
/// * Two equally-fitting models (identical `n`, `RSS`): the one with fewer
///   nodes / constants has the smaller `L_model`, hence the smaller `total`.
/// * Two equally-simple models (identical [`ModelDescription`]): the one with
///   the smaller `RSS` has the smaller `L_data`, hence the smaller `total`.
///
/// # Errors
///
/// Returns [`ScoreError::InvalidDegreesOfFreedom`] when `observations == 0` or
/// `residual_sum_squares` is negative or non-finite, [`ScoreError::InvalidConfig`]
/// when the alphabet is empty, and [`ScoreError::NonFiniteValue`] when a
/// constant value is non-finite.
pub fn description_length(
    observations: usize,
    residual_sum_squares: f64,
    model: &ModelDescription,
) -> Result<DescriptionLength, ScoreError> {
    if observations == 0 || !residual_sum_squares.is_finite() || residual_sum_squares < 0.0 {
        return Err(ScoreError::InvalidDegreesOfFreedom);
    }
    if model.alphabet == 0 {
        return Err(ScoreError::InvalidConfig);
    }
    if model.constants.iter().any(|value| !value.is_finite()) {
        return Err(ScoreError::NonFiniteValue);
    }

    let n = observations as f64;
    let variance = (residual_sum_squares / n).max(f64::MIN_POSITIVE);
    let data_code_length =
        0.5 * n * (TAU * variance).ln() + residual_sum_squares / (2.0 * variance);

    let structural = model.nodes as f64 * (model.alphabet as f64).ln();
    let constant_precision = 0.5 * n.ln();
    let constants: f64 =
        model.constants.iter().map(|value| constant_precision + (1.0 + value.abs()).ln()).sum();
    let model_code_length = structural + constants;

    Ok(DescriptionLength {
        total: data_code_length + model_code_length,
        data_code_length,
        model_code_length,
    })
}

/// Returns the index of the model with the smallest total description length,
/// breaking exact ties by lowest index. Returns `None` for an empty slice.
///
/// This is the MDL analogue of a Pareto filter: instead of a frontier it names
/// the single most-parsimonious candidate under the two-part code.
pub fn most_parsimonious(descriptions: &[DescriptionLength]) -> Option<usize> {
    descriptions
        .iter()
        .enumerate()
        .reduce(|best, current| {
            // total_cmp keeps the choice deterministic even for NaN/±0 inputs;
            // the `is_lt` check plus first-wins reduction gives a stable
            // lowest-index tie-break.
            if current.1.total.total_cmp(&best.1.total).is_lt() { current } else { best }
        })
        .map(|(index, _)| index)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_core::Identifier;
    use lawsynth_expr::Expr;

    fn symbol(name: &str) -> Expr {
        Expr::symbol(Identifier::new(name).unwrap())
    }

    #[test]
    fn from_expression_counts_nodes_operators_and_constants() {
        // x * 2 + 3  ->  nodes: +, *, x, 2, 3  = 5; operators: +, * = 2.
        let expression =
            Expr::sum(Expr::product(symbol("x"), Expr::constant(2.0)), Expr::constant(3.0));
        let description = ModelDescription::from_expression(&expression);
        assert_eq!(description.nodes, 5);
        assert_eq!(description.operators, 2);
        assert_eq!(description.constants, vec![2.0, 3.0]);
        // 10 operators + 1 distinct symbol + 1 constant token.
        assert_eq!(description.alphabet, OPERATOR_VOCABULARY + 2);
    }

    #[test]
    fn prefers_the_simpler_of_two_equally_fitting_models() {
        let simple =
            ModelDescription { nodes: 3, operators: 1, constants: vec![1.0], alphabet: 12 };
        let complex = ModelDescription {
            nodes: 9,
            operators: 4,
            constants: vec![1.0, 2.0, 3.0],
            alphabet: 12,
        };
        let simple_dl = description_length(64, 1.5, &simple).unwrap();
        let complex_dl = description_length(64, 1.5, &complex).unwrap();
        assert_eq!(simple_dl.data_code_length, complex_dl.data_code_length);
        assert!(simple_dl.total < complex_dl.total);
    }

    #[test]
    fn prefers_the_better_fitting_of_two_equally_simple_models() {
        let model = ModelDescription { nodes: 4, operators: 2, constants: vec![0.5], alphabet: 12 };
        let better = description_length(50, 0.25, &model).unwrap();
        let worse = description_length(50, 4.0, &model).unwrap();
        assert_eq!(better.model_code_length, worse.model_code_length);
        assert!(better.total < worse.total);
    }

    #[test]
    fn data_code_length_is_strictly_increasing_in_residual_error() {
        let model = ModelDescription { nodes: 2, operators: 1, constants: vec![], alphabet: 11 };
        let mut previous = f64::NEG_INFINITY;
        for rss in [0.0, 0.1, 1.0, 10.0, 100.0] {
            let value = description_length(32, rss, &model).unwrap().data_code_length;
            assert!(value.is_finite());
            assert!(value > previous, "rss {rss} did not increase the data term");
            previous = value;
        }
    }

    #[test]
    fn constant_magnitude_and_count_raise_the_model_term() {
        let base = ModelDescription { nodes: 3, operators: 1, constants: vec![1.0], alphabet: 12 };
        let large_constant = ModelDescription { constants: vec![1000.0], ..base.clone() };
        let more_constants = ModelDescription { constants: vec![1.0, 1.0], ..base.clone() };
        let base_dl = description_length(20, 1.0, &base).unwrap();
        assert!(description_length(20, 1.0, &large_constant).unwrap().total > base_dl.total);
        assert!(description_length(20, 1.0, &more_constants).unwrap().total > base_dl.total);
    }

    #[test]
    fn most_parsimonious_selects_minimum_with_lowest_index_tiebreak() {
        let model = ModelDescription { nodes: 3, operators: 1, constants: vec![1.0], alphabet: 12 };
        let descriptions = [
            description_length(40, 2.0, &model).unwrap(),
            description_length(40, 0.5, &model).unwrap(),
            description_length(40, 0.5, &model).unwrap(),
        ];
        // Index 1 and 2 tie on the minimum; the lowest index wins.
        assert_eq!(most_parsimonious(&descriptions), Some(1));
        assert_eq!(most_parsimonious(&[]), None);
    }

    #[test]
    fn rejects_invalid_inputs() {
        let model = ModelDescription { nodes: 1, operators: 0, constants: vec![], alphabet: 11 };
        assert_eq!(description_length(0, 1.0, &model), Err(ScoreError::InvalidDegreesOfFreedom));
        assert_eq!(description_length(10, -1.0, &model), Err(ScoreError::InvalidDegreesOfFreedom));
        assert_eq!(
            description_length(10, f64::NAN, &model),
            Err(ScoreError::InvalidDegreesOfFreedom)
        );
        let empty_alphabet = ModelDescription { alphabet: 0, ..model.clone() };
        assert_eq!(description_length(10, 1.0, &empty_alphabet), Err(ScoreError::InvalidConfig));
        let bad_constant = ModelDescription { constants: vec![f64::INFINITY], ..model };
        assert_eq!(description_length(10, 1.0, &bad_constant), Err(ScoreError::NonFiniteValue));
    }

    #[test]
    fn scores_are_bit_for_bit_deterministic() {
        let expression = Expr::sum(
            Expr::product(symbol("x"), Expr::constant(2.5)),
            Expr::difference(symbol("y"), Expr::constant(0.75)),
        );
        let model = ModelDescription::from_expression(&expression);
        let first = description_length(128, 3.5, &model).unwrap();
        let second = description_length(128, 3.5, &model).unwrap();
        assert_eq!(first.total.to_bits(), second.total.to_bits());
        assert_eq!(first.data_code_length.to_bits(), second.data_code_length.to_bits());
        assert_eq!(first.model_code_length.to_bits(), second.model_code_length.to_bits());
    }
}
