use lawsynth_core::Identifier;

/// A single discovered term of the evolution law: `coefficient · uᵖ · D_m`.
#[derive(Clone, Debug, PartialEq)]
pub struct PdeTerm {
    /// A human-readable label such as `u_xx`, `u*u_x`, or `1`.
    pub label: String,
    /// The power `p` of the field in this term (`0` for a pure derivative term).
    pub u_power: usize,
    /// The spatial-derivative order `m` (`0` for the constant / pure-power term).
    pub derivative_order: usize,
    /// The fitted coefficient (exactly `0.0` when the term was thresholded out).
    pub coefficient: f64,
}

/// The discovered evolution law `u_t = Σ coefficient · uᵖ · D_m`.
///
/// Terms are stored in the fixed library order (derivative order outer, field
/// power inner — see [`crate::PdeConfig`]).
#[derive(Clone, Debug, PartialEq)]
pub struct PdeModel {
    /// The field symbol these terms describe.
    pub variable: Identifier,
    /// The candidate terms with their fitted coefficients, in library order.
    pub terms: Vec<PdeTerm>,
    /// Residual sum of squares of the fit against the flattened interior `u_t`,
    /// in the original (un-rescaled) units of `u_t`.
    pub residual_sum_squares: f64,
    /// The spatial step the derivatives were computed with.
    pub dx: f64,
    /// The time step the time derivative was computed with.
    pub dt: f64,
    /// How many interior `(t, x)` points fed the regression (the row count).
    pub interior_points: usize,
    /// The maximum field power in the candidate library.
    pub max_u_degree: usize,
    /// The maximum spatial-derivative order in the candidate library.
    pub max_derivative_order: usize,
}

impl PdeModel {
    /// The coefficient of the term `uᵖ · D_m`, or `0.0` if it is not in the
    /// library. `coefficient_of(0, 2)` fetches the `u_xx` coefficient,
    /// `coefficient_of(1, 1)` the `u·u_x` coefficient.
    pub fn coefficient_of(&self, u_power: usize, derivative_order: usize) -> f64 {
        self.terms
            .iter()
            .find(|term| term.u_power == u_power && term.derivative_order == derivative_order)
            .map(|term| term.coefficient)
            .unwrap_or(0.0)
    }

    /// Looks up a term by its exact label (`"u_xx"`, `"u*u_x"`, ...).
    pub fn term(&self, label: &str) -> Option<&PdeTerm> {
        self.terms.iter().find(|term| term.label == label)
    }

    /// The terms with a non-zero coefficient, in library order.
    pub fn active_terms(&self) -> impl Iterator<Item = &PdeTerm> {
        self.terms.iter().filter(|term| term.coefficient != 0.0)
    }

    /// Renders the discovered law as `u_t = a*u_xx + b*u*u_x` (active terms only).
    ///
    /// Returns `u_t = 0` when every term was thresholded out.
    pub fn describe(&self) -> String {
        let mut active = self.active_terms().peekable();
        if active.peek().is_none() {
            return format!("{}_t = 0", self.variable);
        }
        let body = active
            .map(|term| format!("{:+.6}*{}", term.coefficient, term.label))
            .collect::<Vec<_>>()
            .join(" ");
        format!("{}_t = {body}", self.variable)
    }
}
