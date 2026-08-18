use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, print};

use crate::FeatureTerm;

pub(crate) fn terms(
    variables: &[Identifier],
    degree: usize,
    include_constant: bool,
) -> Vec<FeatureTerm> {
    let mut exponents = vec![0; variables.len()];
    let mut result = Vec::new();
    for total_degree in 0..=degree {
        collect_exponents(
            variables,
            total_degree,
            0,
            &mut exponents,
            include_constant,
            &mut result,
        );
    }
    result
}

fn collect_exponents(
    variables: &[Identifier],
    remaining: usize,
    index: usize,
    exponents: &mut [usize],
    include_constant: bool,
    result: &mut Vec<FeatureTerm>,
) {
    if index + 1 == variables.len() {
        exponents[index] = remaining;
        if include_constant || exponents.iter().any(|exponent| *exponent != 0) {
            let expression = monomial(variables, exponents);
            result.push(FeatureTerm { name: print(&expression), expression });
        }
        return;
    }
    for exponent in 0..=remaining {
        exponents[index] = exponent;
        collect_exponents(
            variables,
            remaining - exponent,
            index + 1,
            exponents,
            include_constant,
            result,
        );
    }
}

fn monomial(variables: &[Identifier], exponents: &[usize]) -> Expr {
    let mut expression = Expr::constant(1.0);
    for (variable, exponent) in variables.iter().zip(exponents) {
        for _ in 0..*exponent {
            expression =
                Expr::binary(BinaryOperator::Multiply, expression, Expr::symbol(variable.clone()));
        }
    }
    expression.simplify()
}
