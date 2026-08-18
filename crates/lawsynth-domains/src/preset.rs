//! The [`DomainPreset`]: a curated, self-validated discovery starting point.
//!
//! A preset bundles four things for one scientific domain:
//!
//! 1. a candidate feature-library configuration (polynomial degree, optional trig
//!    / rational families) tuned so the domain's law is inside the search space;
//! 2. an optional [`TemplatePrior`] encoding a hard structural assumption (e.g.
//!    "no spontaneous source term") that shrinks the candidate set;
//! 3. optional, honestly-expressible SI unit hints for the state variables; and
//! 4. a documented [`ReferenceSystem`] — the canonical governing law plus a
//!    deterministic way to synthesize a clean trajectory from it.
//!
//! The binding property is round-trip recovery: integrating the reference law and
//! running discovery with this preset's own configuration recovers that law. That
//! is asserted by the crate's integration tests, one per preset.

use lawsynth_core::Identifier;
use lawsynth_discovery::{DiscoveryConfig, TemplatePrior};
use lawsynth_features::FeatureConfig;
use lawsynth_units::Unit;

use crate::reference::ReferenceSystem;

/// A SI unit hint attached to a state variable. Kept deliberately light: a preset
/// only attaches a unit it can actually express, and leaves abstract counts or
/// concentrations unannotated rather than inventing a dimension.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitHint {
    /// The annotated state variable.
    pub variable: Identifier,
    /// Its physical unit.
    pub unit: Unit,
}

/// A curated, self-validated domain preset.
///
/// All fields are private; a preset is immutable once built by the catalog. Its
/// configuration is exposed through accessors, and the fully-assembled discovery
/// configuration through [`discovery_config`](Self::discovery_config).
#[derive(Clone, Debug, PartialEq)]
pub struct DomainPreset {
    name: &'static str,
    summary: &'static str,
    reference: ReferenceSystem,
    polynomial_degree: usize,
    include_trigonometric: bool,
    include_rational: bool,
    sparse_threshold: f64,
    template_prior: Option<TemplatePrior>,
    unit_hints: Vec<UnitHint>,
}

impl DomainPreset {
    /// Assembles a preset from its parts. Crate-internal: only the catalog builds
    /// presets, guaranteeing every shipped preset is one the round-trip tests cover.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: &'static str,
        summary: &'static str,
        reference: ReferenceSystem,
        polynomial_degree: usize,
        include_trigonometric: bool,
        include_rational: bool,
        sparse_threshold: f64,
        template_prior: Option<TemplatePrior>,
        unit_hints: Vec<UnitHint>,
    ) -> Self {
        Self {
            name,
            summary,
            reference,
            polynomial_degree,
            include_trigonometric,
            include_rational,
            sparse_threshold,
            template_prior,
            unit_hints,
        }
    }

    /// The stable, lowercase lookup name of the preset (e.g. `"lotka-volterra"`).
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// A one-line human summary of the domain and its reference law.
    pub fn summary(&self) -> &'static str {
        self.summary
    }

    /// The state variables the preset discovers laws for, in reference order.
    pub fn state_variables(&self) -> &[Identifier] {
        self.reference.variables()
    }

    /// The documented reference system (canonical law + trajectory generator).
    pub fn reference(&self) -> &ReferenceSystem {
        &self.reference
    }

    /// The candidate feature-library shape for the domain. The discovery pipeline
    /// always includes the constant intercept, so [`FeatureConfig::include_constant`]
    /// is reported as `true` to mirror what the solver actually sees.
    pub fn feature_config(&self) -> FeatureConfig {
        FeatureConfig { polynomial_degree: self.polynomial_degree, include_constant: true }
    }

    /// The optional structural template prior, if the domain benefits from one.
    pub fn template_prior(&self) -> Option<&TemplatePrior> {
        self.template_prior.as_ref()
    }

    /// The honestly-expressible SI unit hints for the state variables (possibly
    /// empty for abstract, dimensionless domains).
    pub fn unit_hints(&self) -> &[UnitHint] {
        &self.unit_hints
    }

    /// Whether the domain's candidate library includes the sine/cosine family.
    pub fn include_trigonometric(&self) -> bool {
        self.include_trigonometric
    }

    /// Whether the domain's candidate library includes the bounded-rational family.
    pub fn include_rational(&self) -> bool {
        self.include_rational
    }

    /// Assembles the full [`DiscoveryConfig`] the preset runs discovery with:
    /// the state set, feature-library shape, sparse threshold, and — when present
    /// — the template prior. This is exactly the configuration under which the
    /// round-trip recovery is validated.
    pub fn discovery_config(&self) -> DiscoveryConfig {
        let mut config = DiscoveryConfig::new(self.state_variables().iter().cloned());
        config.polynomial_degree = self.polynomial_degree;
        config.include_trigonometric = self.include_trigonometric;
        config.include_rational = self.include_rational;
        config.sparse.threshold = self.sparse_threshold;
        if let Some(prior) = &self.template_prior {
            config.with_template_prior(prior.clone());
        }
        config
    }
}
