use lawsynth_core::Identifier;

/// The fitted sparse dynamics for a single state derivative `ẋ_i`.
///
/// Coefficients are aligned positionally with
/// [`ControlledModel::library_terms`](crate::ControlledModel::library_terms):
/// entry `k` multiplies term `k` of the augmented library `Θ(x, u)`. A zero
/// entry means the sparse solve dropped that term.
#[derive(Clone, Debug, PartialEq)]
pub struct StateEquation {
    /// The state whose derivative this equation predicts.
    pub state: Identifier,
    /// Sparse coefficient row over the augmented library, in library-term order.
    pub coefficients: Vec<f64>,
    /// Residual sum of squares `‖Θ ξ − ẋ‖²` of the fit.
    pub residual_sum_squares: f64,
}

impl StateEquation {
    /// Returns the surviving `(term_label, coefficient)` pairs in library order.
    ///
    /// `labels` must be the model's library terms; only non-zero coefficients
    /// are returned so the result reads as the discovered right-hand side.
    pub fn active_terms<'a>(&self, labels: &'a [String]) -> Vec<(&'a str, f64)> {
        self.coefficients
            .iter()
            .zip(labels)
            .filter(|(coefficient, _)| **coefficient != 0.0)
            .map(|(coefficient, label)| (label.as_str(), *coefficient))
            .collect()
    }
}

/// A discovered controlled dynamical system `ẋ = Θ(x, u) Ξ`.
///
/// The model carries one [`StateEquation`] per state (never per control), the
/// shared augmented-library term labels, and the state/control designation used
/// to build it. Controls appear only inside `library_terms` as inputs; there is
/// deliberately no equation predicting a control.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlledModel {
    /// One fitted equation per state, in the spec's state order.
    pub equations: Vec<StateEquation>,
    /// Human-readable labels for every augmented-library term, in column order.
    pub library_terms: Vec<String>,
    /// State identifiers in the order they were designated.
    pub states: Vec<Identifier>,
    /// Control identifiers in the order they were designated.
    pub controls: Vec<Identifier>,
}

impl ControlledModel {
    /// Looks up the fitted equation for a given state derivative.
    pub fn equation(&self, state: &Identifier) -> Option<&StateEquation> {
        self.equations.iter().find(|equation| &equation.state == state)
    }
}
