use lawsynth_core::Identifier;
use lawsynth_sparse::SparseConfig;

use crate::PdeError;
use crate::derivatives::spatial_half_width;

/// The largest spatial-derivative order the stencils support (`u_xxx`).
pub(crate) const MAX_SUPPORTED_DERIVATIVE_ORDER: usize = 3;

/// Configuration for [`crate::discover_pde`].
///
/// Every field is explicit and deterministic — there is no hidden randomness and
/// no wall-clock input. Identical `(field, dx, dt, PdeConfig)` inputs always
/// produce a bit-identical [`crate::PdeModel`].
#[derive(Clone, Debug, PartialEq)]
pub struct PdeConfig {
    /// The symbol used to label discovered terms (`u`, `c`, ...). Purely
    /// cosmetic — it does not affect the numerics.
    pub variable: Identifier,
    /// The maximum power of the field `u` in a candidate term (`u`, `u²`, ...).
    /// `2` covers the quadratic advective nonlinearity `u·u_x` of Burgers.
    pub max_u_degree: usize,
    /// The maximum spatial-derivative order in a candidate term. `2` covers the
    /// diffusive `u_xx`; the stencils support up to `3` (`u_xxx`).
    pub max_derivative_order: usize,
    /// Whether the library includes the constant intercept `1`.
    pub include_constant: bool,
    /// Sparse-regression settings used to fit the flattened `u_t`. The design
    /// matrix and target are internally rescaled by `RMS(u_t)` before the solve,
    /// so [`SparseConfig::threshold`] acts as a **dimensionless fraction** of the
    /// dominant balance rather than an absolute magnitude.
    pub sparse: SparseConfig,
}

impl Default for PdeConfig {
    fn default() -> Self {
        Self {
            variable: Identifier::new("u").expect("`u` is a valid identifier"),
            max_u_degree: 2,
            max_derivative_order: 2,
            include_constant: true,
            // Threshold is relative (see `sparse` doc): 2% of the dominant term.
            sparse: SparseConfig { threshold: 0.02, max_iterations: 20, ridge: 1e-8 },
        }
    }
}

impl PdeConfig {
    /// A default configuration (variable `u`, degree 2, derivative order 2).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the symbol used to label discovered terms.
    pub fn with_variable(mut self, variable: Identifier) -> Self {
        self.variable = variable;
        self
    }

    /// Sets the maximum power of the field in a candidate term.
    pub fn with_u_degree(mut self, degree: usize) -> Self {
        self.max_u_degree = degree;
        self
    }

    /// Sets the maximum spatial-derivative order in a candidate term.
    pub fn with_derivative_order(mut self, order: usize) -> Self {
        self.max_derivative_order = order;
        self
    }

    /// Includes or drops the constant intercept term `1`.
    pub fn with_constant(mut self, include: bool) -> Self {
        self.include_constant = include;
        self
    }

    /// Overrides the sparse-regression settings.
    pub fn with_sparse(mut self, sparse: SparseConfig) -> Self {
        self.sparse = sparse;
        self
    }

    /// The spatial half-width of the widest stencil the library needs.
    ///
    /// The discovery interior drops this many columns from each spatial edge.
    pub(crate) fn spatial_half_width(&self) -> usize {
        spatial_half_width(self.max_derivative_order)
    }

    /// The number of candidate library columns for this configuration.
    pub(crate) fn library_term_count(&self) -> usize {
        let full = (self.max_u_degree + 1) * (self.max_derivative_order + 1);
        if self.include_constant { full } else { full - 1 }
    }

    /// Validates the numeric fields independent of any field data.
    pub(crate) fn validate(&self) -> Result<(), PdeError> {
        if self.max_derivative_order == 0 {
            return Err(PdeError::InvalidConfig(
                "max_derivative_order must be >= 1 (there is nothing to discover from u alone)"
                    .to_owned(),
            ));
        }
        if self.max_derivative_order > MAX_SUPPORTED_DERIVATIVE_ORDER {
            return Err(PdeError::InvalidConfig(format!(
                "max_derivative_order {} exceeds the supported maximum {MAX_SUPPORTED_DERIVATIVE_ORDER}",
                self.max_derivative_order
            )));
        }
        if self.library_term_count() == 0 {
            return Err(PdeError::InvalidConfig(
                "candidate library would be empty; raise the orders or keep the constant"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}
