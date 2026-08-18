use crate::result::ImplicitRelation;

/// A single monomial `coefficient · ∏ xᵢ^{eᵢ}` of a polynomial.
#[derive(Clone, Debug, PartialEq)]
pub struct MonomialTerm {
    pub name: String,
    pub exponents: Vec<usize>,
    pub coefficient: f64,
}

/// A dense-in-listed-terms polynomial in the state variables.
#[derive(Clone, Debug, PartialEq)]
pub struct Polynomial {
    pub terms: Vec<MonomialTerm>,
}

impl Polynomial {
    /// Evaluates the polynomial at a state vector (identifier-sorted order).
    pub fn evaluate(&self, states: &[f64]) -> f64 {
        self.terms
            .iter()
            .map(|term| {
                let monomial: f64 = term
                    .exponents
                    .iter()
                    .zip(states)
                    .map(|(&exponent, &value)| value.powi(exponent as i32))
                    .product();
                term.coefficient * monomial
            })
            .sum()
    }

    /// Returns the coefficient of the constant term, if present.
    pub fn constant(&self) -> f64 {
        self.terms
            .iter()
            .find(|term| term.exponents.iter().all(|&e| e == 0))
            .map(|term| term.coefficient)
            .unwrap_or(0.0)
    }
}

/// An explicit rational dynamics law `ẋ = P(x) / Q(x)`.
///
/// This is the affine-in-`ẋ` rearrangement of an implicit relation
/// `A(x) + ẋ·B(x) = 0`, giving `P = -A` and `Q = B`. The representation is
/// normalised so the highest-degree denominator term has coefficient `1`.
#[derive(Clone, Debug, PartialEq)]
pub struct RationalLaw {
    pub target: String,
    pub numerator: Polynomial,
    pub denominator: Polynomial,
    /// Whether the denominator stayed away from zero across the samples.
    pub denominator_nonvanishing: bool,
    /// The smallest `|Q(x)|` observed over the fitting samples.
    pub min_abs_denominator: f64,
}

impl RationalLaw {
    /// Evaluates `P(x) / Q(x)` at a state vector.
    pub fn evaluate(&self, states: &[f64]) -> f64 {
        self.numerator.evaluate(states) / self.denominator.evaluate(states)
    }
}

/// Rearranges an implicit relation `A(x) + ẋ·B(x) = 0` into `ẋ = P/Q`.
///
/// Returns `None` when the relation does not actually involve the derivative
/// (`B ≡ 0`), i.e. it is a purely algebraic constraint among the states rather
/// than a dynamics law. The `state_rows` are used only to probe how close the
/// denominator comes to a pole over the observed data.
pub(crate) fn reconstruct(
    relation: &ImplicitRelation,
    target: &str,
    state_rows: &[Vec<f64>],
    min_denominator: f64,
) -> Option<RationalLaw> {
    const DROP: f64 = 1e-9;

    let mut numerator = Vec::new();
    let mut denominator = Vec::new();
    for term in &relation.terms {
        if term.coefficient.abs() < DROP {
            continue;
        }
        if term.term.involves_derivative {
            // B(x): denominator, Q = B.
            denominator.push(MonomialTerm {
                name: monomial_label(&term.term.exponents, &term.term.name),
                exponents: term.term.exponents.clone(),
                coefficient: term.coefficient,
            });
        } else {
            // A(x): numerator, P = -A.
            numerator.push(MonomialTerm {
                name: term.term.name.clone(),
                exponents: term.term.exponents.clone(),
                coefficient: -term.coefficient,
            });
        }
    }
    if denominator.is_empty() {
        return None;
    }

    // Normalise so the highest-degree denominator term is monic; fall back to
    // the largest-magnitude coefficient when the leading one is numerically
    // negligible. This is deterministic and yields the canonical `Q = Km + x`
    // form for Michaelis-Menten.
    let norm = leading_coefficient(&denominator);
    if norm.abs() < DROP {
        return None;
    }
    for term in numerator.iter_mut().chain(denominator.iter_mut()) {
        term.coefficient /= norm;
    }

    let denominator = Polynomial { terms: denominator };
    let min_abs_denominator = state_rows
        .iter()
        .map(|states| denominator.evaluate(states).abs())
        .fold(f64::INFINITY, f64::min);
    let min_abs_denominator =
        if min_abs_denominator.is_finite() { min_abs_denominator } else { 0.0 };

    Some(RationalLaw {
        target: target.to_string(),
        numerator: Polynomial { terms: numerator },
        denominator,
        denominator_nonvanishing: min_abs_denominator > min_denominator,
        min_abs_denominator,
    })
}

/// Picks the coefficient of the highest-degree denominator term, breaking ties
/// by largest magnitude and then by term listing order.
fn leading_coefficient(terms: &[MonomialTerm]) -> f64 {
    terms
        .iter()
        .enumerate()
        .max_by(|(left_index, left), (right_index, right)| {
            let left_degree: usize = left.exponents.iter().sum();
            let right_degree: usize = right.exponents.iter().sum();
            left_degree
                .cmp(&right_degree)
                .then(left.coefficient.abs().total_cmp(&right.coefficient.abs()))
                .then(right_index.cmp(left_index))
        })
        .map(|(_, term)| term.coefficient)
        .unwrap_or(0.0)
}

/// Strips a trailing derivative factor from a term name so denominator terms
/// read as plain polynomials in the states (`"x*dx"` -> `"x"`, `"dx"` -> `"1"`).
fn monomial_label(exponents: &[usize], derivative_name: &str) -> String {
    if exponents.iter().all(|&e| e == 0) {
        "1".to_string()
    } else {
        // Reuse the state part of the derivative term name (everything before
        // the final `*d...` factor).
        match derivative_name.rsplit_once('*') {
            Some((base, _)) => base.to_string(),
            None => "1".to_string(),
        }
    }
}
