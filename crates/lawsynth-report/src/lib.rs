//! Self-contained HTML report rendering for LawSynth executable worlds.
//!
//! Given a [`World`], [`render_report`] simulates a default forward trajectory
//! and produces a single, dependency-free HTML document with rendered law
//! equations, variable/parameter tables, and hand-built inline SVG charts
//! (a trajectory line chart plus a phase portrait for multi-state worlds).

mod html;
mod render;
mod svg;

use std::collections::BTreeMap;
use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory, simulate};
use lawsynth_world::World;

pub use render::{
    format_number, python_number, render_continuous_law, render_discrete_law, render_expression,
    render_latex_expression, render_latex_law, render_python_expression,
};
pub use svg::RegimeSpan;

mod comparison;
pub use comparison::render_comparison;

/// Observed samples overlaid on a report to show fit quality.
///
/// Each column is aligned to the shared [`time`](Self::time) axis and keyed by
/// the state identifier it measures. When present, the report renders a fit
/// overlay (simulated vs observed) and a residual strip.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ReportObservations {
    /// Observation timestamps shared by every column.
    pub time: Vec<f64>,
    /// Observed samples keyed by state identifier.
    pub columns: BTreeMap<Identifier, Vec<f64>>,
}

/// A per-state uncertainty envelope rendered as a band + median line.
#[derive(Clone, Debug, PartialEq)]
pub struct UncertaintyBand {
    /// State the band describes.
    pub state: Identifier,
    /// Shared time axis for all three envelopes.
    pub time: Vec<f64>,
    /// Lower envelope.
    pub lower: Vec<f64>,
    /// Median trajectory.
    pub median: Vec<f64>,
    /// Upper envelope.
    pub upper: Vec<f64>,
}

/// Rendering and simulation options for a world report.
#[derive(Clone, Debug, PartialEq)]
pub struct ReportOptions {
    /// Document title shown in the report header.
    pub title: String,
    /// Inclusive simulation start time.
    pub start: f64,
    /// Inclusive simulation end time.
    pub end: f64,
    /// Maximum integration step.
    pub step: f64,
    /// Initial value used for any state without an explicit override.
    pub default_initial: f64,
    /// Per-state initial-value overrides.
    pub initial_overrides: BTreeMap<Identifier, f64>,
    /// Optional observed data overlaid to show fit quality (residual view).
    pub observations: Option<ReportObservations>,
    /// Optional discovered regime spans rendered as a timeline.
    pub regimes: Option<Vec<RegimeSpan>>,
    /// Optional per-state uncertainty bands.
    pub uncertainty: Option<Vec<UncertaintyBand>>,
}

impl Default for ReportOptions {
    fn default() -> Self {
        Self {
            title: "LawSynth World Report".to_owned(),
            start: 0.0,
            end: 10.0,
            step: 0.1,
            default_initial: 1.0,
            initial_overrides: BTreeMap::new(),
            observations: None,
            regimes: None,
            uncertainty: None,
        }
    }
}

/// A failure while building a report.
#[derive(Debug)]
pub enum ReportError {
    /// The default trajectory could not be simulated.
    Simulation(String),
    /// The simulation configuration was invalid.
    Configuration(String),
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simulation(message) => write!(f, "simulation failed: {message}"),
            Self::Configuration(message) => write!(f, "invalid report configuration: {message}"),
        }
    }
}

impl std::error::Error for ReportError {}

/// Builds the default forward trajectory used by a report.
///
/// Exposed so callers (e.g. the CLI) can reuse the same deterministic
/// simulation the report renders.
pub fn simulate_default(world: &World, options: &ReportOptions) -> Result<Trajectory, ReportError> {
    let config = SimulationConfig::new(options.start, options.end, options.step)
        .map_err(|error| ReportError::Configuration(error.to_string()))?;
    let mut request = SimulationRequest::default();
    for state in world.state_ids() {
        let value =
            options.initial_overrides.get(state).copied().unwrap_or(options.default_initial);
        request = request.with_initial(state.clone(), value);
    }
    simulate(world, config, &request).map_err(|error| ReportError::Simulation(error.to_string()))
}

/// Renders a complete standalone HTML report for a world.
pub fn render_report(world: &World, options: &ReportOptions) -> Result<String, ReportError> {
    let trajectory = simulate_default(world, options)?;
    Ok(html::page(&options.title, world, &trajectory, options))
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn decay_world() -> World {
        World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
            )],
        )
        .unwrap()
    }

    #[test]
    fn renders_a_self_contained_document() {
        let html = render_report(&decay_world(), &ReportOptions::default()).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<svg"));
        // No external assets.
        assert!(!html.contains("src=\"http"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn is_deterministic() {
        let world = decay_world();
        let options = ReportOptions::default();
        assert_eq!(
            render_report(&world, &options).unwrap(),
            render_report(&world, &options).unwrap()
        );
    }
}
