//! The forecast bands produced by both propagation methods.

use std::fmt::Write as _;

use lawsynth_core::Identifier;

/// A forecast with prediction bands: for every state and every time sample, a
/// central estimate, a variance, and lower/upper band endpoints.
///
/// All time-indexed collections share the trajectory's time grid. Each of the
/// four value matrices is indexed `[state][time]`: `mean()[i][k]` is the central
/// forecast of state `i` at `times()[k]`, `variance()[i][k]` its propagated
/// variance, and `lower()[i][k]` / `upper()[i][k]` the band endpoints. `states()`
/// gives the identifier ordering that indexes the first axis.
///
/// The two methods fill these fields with the same shape but different content:
///
/// - **Delta method** — `mean` is the nominal trajectory `x(t)` integrated at the
///   supplied parameters, `variance` is `diag(S(t)·Cov(θ)·S(t)ᵀ)`, and the band is
///   `mean ± z·sqrt(variance)`, a symmetric first-order Gaussian band.
/// - **Monte-Carlo** — `mean` is the per-time empirical mean over the ensemble of
///   simulated trajectories, `variance` the empirical (unbiased) variance, and the
///   band the empirical lower/upper percentiles; it is generally asymmetric.
#[derive(Clone, Debug, PartialEq)]
pub struct ForecastBands {
    times: Vec<f64>,
    states: Vec<Identifier>,
    mean: Vec<Vec<f64>>,
    variance: Vec<Vec<f64>>,
    lower: Vec<Vec<f64>>,
    upper: Vec<Vec<f64>>,
}

impl ForecastBands {
    pub(crate) fn new(
        times: Vec<f64>,
        states: Vec<Identifier>,
        mean: Vec<Vec<f64>>,
        variance: Vec<Vec<f64>>,
        lower: Vec<Vec<f64>>,
        upper: Vec<Vec<f64>>,
    ) -> Self {
        Self { times, states, mean, variance, lower, upper }
    }

    /// The shared time grid.
    pub fn times(&self) -> &[f64] {
        &self.times
    }

    /// The state ordering that indexes the first axis of every value matrix.
    pub fn states(&self) -> &[Identifier] {
        &self.states
    }

    /// The number of time samples.
    pub fn sample_count(&self) -> usize {
        self.times.len()
    }

    /// The state-space dimension.
    pub fn dimension(&self) -> usize {
        self.states.len()
    }

    /// The central forecast, `mean()[state][time]`.
    pub fn mean(&self) -> &[Vec<f64>] {
        &self.mean
    }

    /// The propagated variance, `variance()[state][time]`.
    pub fn variance(&self) -> &[Vec<f64>] {
        &self.variance
    }

    /// The lower band endpoint, `lower()[state][time]`.
    pub fn lower(&self) -> &[Vec<f64>] {
        &self.lower
    }

    /// The upper band endpoint, `upper()[state][time]`.
    pub fn upper(&self) -> &[Vec<f64>] {
        &self.upper
    }

    /// The band width `upper − lower` for a `(state, time)` index, or `None` if
    /// either index is out of range.
    pub fn band_width(&self, state: usize, time: usize) -> Option<f64> {
        let upper = self.upper.get(state)?.get(time)?;
        let lower = self.lower.get(state)?.get(time)?;
        Some(upper - lower)
    }

    /// A stable textual fingerprint encoding every float by its `f64` bit
    /// pattern. Two runs on identical input MUST produce identical strings; this
    /// is the basis of the determinism guarantee.
    pub fn to_canonical_string(&self) -> String {
        let mut output = String::new();
        output.push_str("states:");
        for state in &self.states {
            output.push_str(state.as_str());
            output.push(',');
        }
        output.push('\n');
        for (step, time) in self.times.iter().enumerate() {
            let _ = write!(output, "t={:016x}", time.to_bits());
            for state in 0..self.states.len() {
                let _ = write!(
                    output,
                    " s{state}[m={:016x},v={:016x},lo={:016x},hi={:016x}]",
                    self.mean[state][step].to_bits(),
                    self.variance[state][step].to_bits(),
                    self.lower[state][step].to_bits(),
                    self.upper[state][step].to_bits(),
                );
            }
            output.push('\n');
        }
        output
    }
}
