use crate::InvariantError;

/// Configuration for a conserved-quantity search.
///
/// Every field is deterministic: the same configuration always produces the
/// same candidate library, the same sample grid, and hence the same report.
#[derive(Clone, Debug, PartialEq)]
pub struct InvariantConfig {
    /// Maximum total degree of the monomial basis (the constant, degree 0, is
    /// always excluded). Must be at least 1.
    pub degree: usize,
    /// When true, the library additionally includes `sin(x_i)` and `cos(x_i)`
    /// for every state `x_i`, enabling recovery of simple trigonometric
    /// invariants (e.g. the pendulum energy). A purely polynomial library
    /// (this flag false) cannot express transcendental invariants.
    pub include_trigonometric: bool,
    /// Lower corner of the axis-aligned sample box, shared by every axis.
    pub sample_lo: f64,
    /// Upper corner of the axis-aligned sample box, shared by every axis.
    pub sample_hi: f64,
    /// Number of grid points along each axis. The full grid is the tensor
    /// product, so `resolution^n` points for `n` states. Must be at least 2.
    pub resolution: usize,
    /// A right-singular vector is treated as a conserved quantity when its
    /// singular value `σ` satisfies `σ <= tolerance * σ_max`. Smaller values
    /// reduce false positives at the cost of missing weakly-resolved
    /// invariants.
    pub tolerance: f64,
}

impl InvariantConfig {
    /// Builds a configuration, deferring validation to [`InvariantConfig::validate`].
    pub fn new(
        degree: usize,
        include_trigonometric: bool,
        sample_lo: f64,
        sample_hi: f64,
        resolution: usize,
        tolerance: f64,
    ) -> Self {
        Self { degree, include_trigonometric, sample_lo, sample_hi, resolution, tolerance }
    }

    /// Rejects degenerate configurations with a typed error.
    pub fn validate(&self) -> Result<(), InvariantError> {
        if self.degree == 0 {
            return Err(InvariantError::InvalidDegree);
        }
        if self.resolution < 2 {
            return Err(InvariantError::InvalidResolution);
        }
        if !self.sample_lo.is_finite()
            || !self.sample_hi.is_finite()
            || self.sample_lo >= self.sample_hi
        {
            return Err(InvariantError::InvalidBox);
        }
        if !self.tolerance.is_finite() || self.tolerance < 0.0 {
            return Err(InvariantError::InvalidTolerance);
        }
        Ok(())
    }
}

impl Default for InvariantConfig {
    /// A general-purpose default: quadratic polynomial library, no trig terms,
    /// a mildly asymmetric box `[-1, 1.5]` sampled at 5 points per axis, and a
    /// relative singular-value tolerance of `1e-9`.
    fn default() -> Self {
        Self {
            degree: 2,
            include_trigonometric: false,
            sample_lo: -1.0,
            sample_hi: 1.5,
            resolution: 5,
            tolerance: 1e-9,
        }
    }
}
