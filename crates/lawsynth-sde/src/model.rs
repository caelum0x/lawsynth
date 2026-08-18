use lawsynth_core::Identifier;

use crate::BinRule;

/// One bin of the Kramers–Moyal conditional-moment estimate for a single state.
///
/// `drift` and `diffusion` are the *raw* binned estimates before any sparse fit:
///
/// ```text
/// drift(x)     ≈ E[ΔX  | X ∈ bin] / Δt   (1st Kramers–Moyal coefficient)
/// diffusion(x) ≈ E[ΔX² | X ∈ bin] / Δt   (2nd Kramers–Moyal coefficient, i.e. b²)
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BinnedEstimate {
    /// The occupancy-weighted mean of the source states falling in this bin.
    pub x_center: f64,
    /// The estimated drift `a(x)` at `x_center`.
    pub drift: f64,
    /// The estimated diffusion `b²(x)` at `x_center` (a variance rate, ≥ 0).
    pub diffusion: f64,
    /// How many increments contributed to this bin.
    pub count: usize,
}

/// A single term of a discovered closed-form law: `coefficient · variable^power`.
#[derive(Clone, Debug, PartialEq)]
pub struct LawTerm {
    /// A human-readable label such as `1`, `x`, or `x^2`.
    pub label: String,
    /// The monomial power of the state variable (`0` is the constant term).
    pub power: u32,
    /// The fitted coefficient (exactly `0.0` when the term was thresholded out).
    pub coefficient: f64,
}

/// A sparse-regressed closed-form law over the polynomial candidate library.
///
/// Terms are stored in ascending power order, matching the candidate library.
#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveredLaw {
    pub terms: Vec<LawTerm>,
    /// Residual sum of squares of the fit against the trusted binned estimates.
    pub residual_sum_squares: f64,
}

impl DiscoveredLaw {
    /// Evaluates the fitted polynomial law at `x`.
    pub fn evaluate(&self, x: f64) -> f64 {
        self.terms.iter().map(|term| term.coefficient * x.powi(term.power as i32)).sum()
    }

    /// The coefficient attached to a given monomial power, or `0.0` if absent.
    pub fn coefficient_of_power(&self, power: u32) -> f64 {
        self.terms
            .iter()
            .find(|term| term.power == power)
            .map(|term| term.coefficient)
            .unwrap_or(0.0)
    }

    /// The terms with a non-zero coefficient, in ascending power order.
    pub fn active_terms(&self) -> impl Iterator<Item = &LawTerm> {
        self.terms.iter().filter(|term| term.coefficient != 0.0)
    }
}

/// The discovered drift and diffusion for one state variable.
#[derive(Clone, Debug, PartialEq)]
pub struct StateModel {
    /// The state column this model describes.
    pub state: Identifier,
    /// The raw binned conditional-moment table, in ascending bin order. Only
    /// bins with at least one increment are listed.
    pub bins: Vec<BinnedEstimate>,
    /// How many of `bins` met `min_bin_count` and were used for the sparse fits.
    pub trusted_bins: usize,
    /// The sparse-regressed drift law `a(x)`.
    pub drift_law: DiscoveredLaw,
    /// The sparse-regressed diffusion law `b²(x)`.
    pub diffusion_law: DiscoveredLaw,
}

/// The full result of [`crate::discover_sde`].
#[derive(Clone, Debug, PartialEq)]
pub struct SdeModel {
    /// One entry per estimated state, in the dataset's schema order.
    pub states: Vec<StateModel>,
    /// The mean sampling interval `Δt` used to normalise the moments.
    pub dt: f64,
    /// The bin rule that was applied.
    pub bin_rule: BinRule,
    /// The number of increments (`rows − 1`) processed.
    pub increment_count: usize,
}

impl SdeModel {
    /// Looks up the model for a named state.
    pub fn state(&self, id: &Identifier) -> Option<&StateModel> {
        self.states.iter().find(|state| &state.state == id)
    }
}
