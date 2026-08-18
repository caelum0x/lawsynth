use crate::ImplicitError;

/// A single named column of the augmented library `Θ(x, ẋ)`.
///
/// Each term is a state monomial `∏ xᵢ^{eᵢ}` optionally multiplied by the
/// target derivative `ẋ`. Because every derivative-bearing term carries `ẋ` to
/// the first power, any relation over this library is affine in `ẋ` and can be
/// rearranged into an explicit rational law `ẋ = P(x) / Q(x)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AugmentedTerm {
    /// Human-readable label such as `"1"`, `"x"`, `"x^2"`, `"dx"`, `"x*dx"`.
    pub name: String,
    /// Exponent of each state variable, in identifier-sorted order.
    pub exponents: Vec<usize>,
    /// Whether the state monomial is multiplied by the target derivative `ẋ`.
    pub involves_derivative: bool,
}

/// A deterministic augmented candidate library over states and the derivative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AugmentedLibrary {
    state_names: Vec<String>,
    target: String,
    terms: Vec<AugmentedTerm>,
}

/// The evaluated augmented design matrix, one row per retained sample.
#[derive(Clone, Debug, PartialEq)]
pub struct AugmentedMatrix {
    pub terms: Vec<AugmentedTerm>,
    pub rows: Vec<Vec<f64>>,
}

impl AugmentedLibrary {
    /// Builds `Θ(x, ẋ)`: every state monomial up to `degree`, plus every such
    /// monomial multiplied by `ẋ`. The pure-state monomials come first, in
    /// ascending total-degree then lexicographic order, followed by the
    /// derivative-bearing block in the same order.
    pub fn build(
        state_names: &[String],
        target: &str,
        degree: usize,
        include_constant: bool,
    ) -> Result<Self, ImplicitError> {
        if state_names.is_empty() || degree == 0 {
            return Err(ImplicitError::InvalidConfig);
        }
        let monomials = monomial_exponents(state_names.len(), degree);
        let mut terms = Vec::new();
        for exponents in &monomials {
            let is_constant = exponents.iter().all(|&e| e == 0);
            if is_constant && !include_constant {
                continue;
            }
            terms.push(AugmentedTerm {
                name: monomial_name(exponents, state_names),
                exponents: exponents.clone(),
                involves_derivative: false,
            });
        }
        for exponents in &monomials {
            terms.push(AugmentedTerm {
                name: derivative_name(exponents, state_names, target),
                exponents: exponents.clone(),
                involves_derivative: true,
            });
        }
        Ok(Self { state_names: state_names.to_vec(), target: target.to_string(), terms })
    }

    pub fn terms(&self) -> &[AugmentedTerm] {
        &self.terms
    }

    pub fn state_names(&self) -> &[String] {
        &self.state_names
    }

    pub fn target(&self) -> &str {
        &self.target
    }

    /// Evaluates the library over aligned state rows and derivative samples.
    ///
    /// `state_rows[t][i]` is state `i` at sample `t`; `derivative[t]` is the
    /// estimated `ẋ` of the target at sample `t`.
    pub fn evaluate(
        &self,
        state_rows: &[Vec<f64>],
        derivative: &[f64],
    ) -> Result<AugmentedMatrix, ImplicitError> {
        if state_rows.len() != derivative.len() {
            return Err(ImplicitError::InsufficientSamples);
        }
        let rows = state_rows
            .iter()
            .zip(derivative)
            .map(|(states, &xdot)| {
                self.terms
                    .iter()
                    .map(|term| {
                        let base = monomial_value(&term.exponents, states);
                        let value = if term.involves_derivative { base * xdot } else { base };
                        if value.is_finite() {
                            Ok(value)
                        } else {
                            Err(ImplicitError::NonFiniteValue)
                        }
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(AugmentedMatrix { terms: self.terms.clone(), rows })
    }
}

/// Enumerates every exponent vector over `states` variables whose entries sum
/// to at most `degree`, ordered by ascending total degree then lexicographic
/// order. This ordering is fixed, so the library is bit-for-bit reproducible.
fn monomial_exponents(states: usize, degree: usize) -> Vec<Vec<usize>> {
    let mut result = Vec::new();
    for total in 0..=degree {
        collect_compositions(states, total, &mut vec![0; states], 0, &mut result);
    }
    result
}

fn collect_compositions(
    states: usize,
    remaining: usize,
    current: &mut Vec<usize>,
    position: usize,
    out: &mut Vec<Vec<usize>>,
) {
    if position + 1 == states {
        current[position] = remaining;
        out.push(current.clone());
        return;
    }
    // Assign the largest share to the earliest variable first so that, at a
    // fixed total degree, the vectors come out in descending lexicographic
    // order on the leading variable (a stable, documented convention).
    for value in (0..=remaining).rev() {
        current[position] = value;
        collect_compositions(states, remaining - value, current, position + 1, out);
    }
}

fn monomial_value(exponents: &[usize], states: &[f64]) -> f64 {
    exponents.iter().zip(states).map(|(&exponent, &value)| value.powi(exponent as i32)).product()
}

fn monomial_name(exponents: &[usize], names: &[String]) -> String {
    let factors = exponents
        .iter()
        .zip(names)
        .filter(|&(&exponent, _)| exponent > 0)
        .map(
            |(&exponent, name)| {
                if exponent == 1 { name.clone() } else { format!("{name}^{exponent}") }
            },
        )
        .collect::<Vec<_>>();
    if factors.is_empty() { "1".to_string() } else { factors.join("*") }
}

fn derivative_name(exponents: &[usize], names: &[String], target: &str) -> String {
    let derivative = format!("d{target}");
    let base = monomial_name(exponents, names);
    if base == "1" { derivative } else { format!("{base}*{derivative}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_state_library_is_ordered_and_named() {
        let library = AugmentedLibrary::build(&["x".to_string()], "x", 2, true).unwrap();
        let names = library.terms().iter().map(|t| t.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, vec!["1", "x", "x^2", "dx", "x*dx", "x^2*dx"]);
        assert!(!library.terms()[1].involves_derivative);
        assert!(library.terms()[3].involves_derivative);
    }

    #[test]
    fn dropping_the_constant_keeps_the_bare_derivative() {
        let library = AugmentedLibrary::build(&["x".to_string()], "x", 1, false).unwrap();
        let names = library.terms().iter().map(|t| t.name.as_str()).collect::<Vec<_>>();
        assert_eq!(names, vec!["x", "dx", "x*dx"]);
    }

    #[test]
    fn evaluates_state_and_derivative_columns() {
        let library = AugmentedLibrary::build(&["x".to_string()], "x", 2, true).unwrap();
        let matrix = library.evaluate(&[vec![3.0]], &[2.0]).unwrap();
        // [1, x, x^2, dx, x*dx, x^2*dx] at x=3, dx=2.
        assert_eq!(matrix.rows[0], vec![1.0, 3.0, 9.0, 2.0, 6.0, 18.0]);
    }

    #[test]
    fn two_state_monomials_are_deterministic() {
        let names = vec!["x".to_string(), "y".to_string()];
        let library = AugmentedLibrary::build(&names, "x", 2, true).unwrap();
        let poly = library
            .terms()
            .iter()
            .filter(|t| !t.involves_derivative)
            .map(|t| t.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(poly, vec!["1", "x", "y", "x^2", "x*y", "y^2"]);
    }
}
