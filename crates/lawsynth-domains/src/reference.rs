//! The documented reference system behind a preset: a polynomial vector field,
//! its canonical governing law, and a deterministic fixed-step RK4 trajectory.
//!
//! Every reference law LawSynth ships as a preset is a polynomial vector field
//! `d(state)/dt = f(state)`, where each component of `f` is a sum of
//! coefficient-weighted monomials in the state variables. Representing the law
//! this way lets a single description serve two masters: it is *integrated* by the
//! RK4 stepper to synthesize a clean trajectory, and it is *evaluated* directly
//! by the round-trip test to check that discovery recovered the same polynomial.
//!
//! The integrator reads no clock and draws no randomness: given the same initial
//! condition, step size, and step count it produces a bit-identical trajectory.

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

/// A single monomial `coefficient · ∏ᵢ varᵢ^exponentᵢ`, with the exponent vector
/// aligned to the state-variable order of the owning [`ReferenceSystem`].
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceTerm {
    /// The scalar coefficient multiplying the monomial.
    pub coefficient: f64,
    /// Exponent of each state variable, positionally aligned with
    /// [`ReferenceSystem::variables`].
    pub exponents: Vec<u32>,
}

impl ReferenceTerm {
    /// Builds a monomial term from a coefficient and per-variable exponents.
    pub fn new(coefficient: f64, exponents: impl Into<Vec<u32>>) -> Self {
        Self { coefficient, exponents: exponents.into() }
    }

    /// Evaluates the monomial at a state vector in variable order.
    fn evaluate(&self, state: &[f64]) -> f64 {
        let mut value = self.coefficient;
        for (variable, &exponent) in state.iter().zip(&self.exponents) {
            for _ in 0..exponent {
                value *= *variable;
            }
        }
        value
    }

    /// Total polynomial degree of the monomial (the sum of its exponents).
    pub fn total_degree(&self) -> u32 {
        self.exponents.iter().sum()
    }
}

/// The governing law `d(target)/dt = Σ terms` for one state variable, as a sum of
/// monomial [`ReferenceTerm`]s.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceLaw {
    terms: Vec<ReferenceTerm>,
}

impl ReferenceLaw {
    /// Builds a law from its monomial terms.
    pub fn new(terms: impl Into<Vec<ReferenceTerm>>) -> Self {
        Self { terms: terms.into() }
    }

    /// The monomial terms of the law, in declaration order.
    pub fn terms(&self) -> &[ReferenceTerm] {
        &self.terms
    }

    /// Evaluates the right-hand side at a state vector in variable order.
    fn evaluate(&self, state: &[f64]) -> f64 {
        self.terms.iter().map(|term| term.evaluate(state)).sum()
    }

    /// Number of terms carrying a non-zero coefficient — the count a discovered
    /// law must reproduce for its structure to match.
    pub fn active_terms(&self) -> usize {
        self.terms.iter().filter(|term| term.coefficient != 0.0).count()
    }
}

/// A reference dynamical system: state variables, one polynomial law per
/// variable, a fixed initial condition, and a fixed RK4 schedule.
///
/// Constructed only inside this crate (by the preset catalog) so that the field
/// invariant — `variables`, `laws`, and `initial` share one length and one
/// ordering — always holds.
#[derive(Clone, Debug, PartialEq)]
pub struct ReferenceSystem {
    variables: Vec<Identifier>,
    laws: Vec<ReferenceLaw>,
    initial: Vec<f64>,
    dt: f64,
    steps: usize,
}

impl ReferenceSystem {
    /// Builds a reference system. Panics if the variable, law, and initial-state
    /// lengths disagree or the schedule is degenerate — these are crate-internal
    /// authoring errors, never runtime input.
    pub(crate) fn new(
        variables: Vec<Identifier>,
        laws: Vec<ReferenceLaw>,
        initial: Vec<f64>,
        dt: f64,
        steps: usize,
    ) -> Self {
        assert_eq!(variables.len(), laws.len(), "one law per state variable");
        assert_eq!(variables.len(), initial.len(), "one initial value per state variable");
        assert!(!variables.is_empty(), "a reference system needs at least one state variable");
        assert!(dt > 0.0 && dt.is_finite(), "step size must be positive and finite");
        assert!(steps >= 2, "a usable trajectory needs at least three samples");
        Self { variables, laws, initial, dt, steps }
    }

    /// The ordered state variables of the system.
    pub fn variables(&self) -> &[Identifier] {
        &self.variables
    }

    /// The governing law for `target`, or `None` if it is not a state variable.
    pub fn law(&self, target: &Identifier) -> Option<&ReferenceLaw> {
        self.variables.iter().position(|variable| variable == target).map(|index| &self.laws[index])
    }

    /// The fixed integration step size.
    pub fn dt(&self) -> f64 {
        self.dt
    }

    /// The number of RK4 steps taken (the trajectory has `steps + 1` samples).
    pub fn steps(&self) -> usize {
        self.steps
    }

    /// The fixed initial state, in variable order.
    pub fn initial(&self) -> &[f64] {
        &self.initial
    }

    /// Evaluates `d(target)/dt` from the reference law at `state` (variable order),
    /// or `None` if `target` is not a state variable. This is the ground truth the
    /// round-trip test compares a discovered law against.
    pub fn evaluate_law(&self, target: &Identifier, state: &[f64]) -> Option<f64> {
        self.law(target).map(|law| law.evaluate(state))
    }

    /// Writes `f(state)` into `out` (both in variable order).
    fn rhs(&self, state: &[f64], out: &mut [f64]) {
        for (law, slot) in self.laws.iter().zip(out.iter_mut()) {
            *slot = law.evaluate(state);
        }
    }

    /// One classical fixed-step RK4 step, returning the next state.
    fn rk4_step(&self, state: &[f64]) -> Vec<f64> {
        let dimension = state.len();
        let mut k1 = vec![0.0; dimension];
        self.rhs(state, &mut k1);

        let mut stage = vec![0.0; dimension];
        for index in 0..dimension {
            stage[index] = state[index] + 0.5 * self.dt * k1[index];
        }
        let mut k2 = vec![0.0; dimension];
        self.rhs(&stage, &mut k2);

        for index in 0..dimension {
            stage[index] = state[index] + 0.5 * self.dt * k2[index];
        }
        let mut k3 = vec![0.0; dimension];
        self.rhs(&stage, &mut k3);

        for index in 0..dimension {
            stage[index] = state[index] + self.dt * k3[index];
        }
        let mut k4 = vec![0.0; dimension];
        self.rhs(&stage, &mut k4);

        (0..dimension)
            .map(|index| {
                state[index]
                    + self.dt * (k1[index] + 2.0 * k2[index] + 2.0 * k3[index] + k4[index]) / 6.0
            })
            .collect()
    }

    /// Integrates the reference law into a clean [`Dataset`] via fixed-step RK4.
    ///
    /// The result is a pure function of the system's fixed initial condition,
    /// step size, and step count — identical calls yield a bit-identical dataset.
    pub fn trajectory(&self) -> Dataset {
        let mut series: Vec<Vec<f64>> = self
            .initial
            .iter()
            .map(|&value| {
                let mut channel = Vec::with_capacity(self.steps + 1);
                channel.push(value);
                channel
            })
            .collect();

        let mut state = self.initial.clone();
        for _ in 0..self.steps {
            state = self.rk4_step(&state);
            for (channel, &value) in series.iter_mut().zip(&state) {
                channel.push(value);
            }
        }

        let time = (0..=self.steps).map(|step| step as f64 * self.dt).collect::<Vec<_>>();
        let columns = self
            .variables
            .iter()
            .cloned()
            .zip(series)
            .map(|(id, values)| NumericColumn::new(id, values))
            .collect::<Vec<_>>();
        Dataset::new(
            TimeAxis::new(time).expect("uniform positive step yields a valid time axis"),
            columns,
        )
        .expect("reference trajectories are finite and well-formed by construction")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ident(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    /// Exponential decay `dx/dt = -x`, whose closed form is `x(t) = x0 · e^(-t)`.
    fn decay() -> ReferenceSystem {
        ReferenceSystem::new(
            vec![ident("x")],
            vec![ReferenceLaw::new([ReferenceTerm::new(-1.0, [1])])],
            vec![1.0],
            0.01,
            500,
        )
    }

    #[test]
    fn rk4_matches_the_analytic_decay_solution() {
        let system = decay();
        let data = system.trajectory();
        let x = data.columns()[&ident("x")].values.clone();
        assert_eq!(x.len(), 501);
        let final_time = system.steps() as f64 * system.dt();
        let expected = (-final_time).exp();
        assert!((x.last().unwrap() - expected).abs() < 1e-6, "got {}", x.last().unwrap());
    }

    #[test]
    fn trajectory_is_bit_identical_across_calls() {
        let system = decay();
        let first = system.trajectory();
        let second = system.trajectory();
        let a = &first.columns()[&ident("x")].values;
        let b = &second.columns()[&ident("x")].values;
        assert!(a.iter().zip(b).all(|(left, right)| left.to_bits() == right.to_bits()));
    }

    #[test]
    fn evaluate_law_returns_the_polynomial_right_hand_side() {
        let system = decay();
        assert_eq!(system.evaluate_law(&ident("x"), &[3.0]), Some(-3.0));
        assert_eq!(system.evaluate_law(&ident("missing"), &[3.0]), None);
    }
}
