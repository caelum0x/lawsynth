//! Self-contained HTML report rendering for LawSynth executable worlds.
//!
//! Given a [`World`], [`render_report`] simulates a default forward trajectory
//! and produces a single, dependency-free HTML document with rendered law
//! equations, variable/parameter tables, and hand-built inline SVG charts
//! (a trajectory line chart plus a phase portrait for multi-state worlds).

mod analysis;
mod html;
mod render;
mod svg;
mod theme;

pub use theme::Theme;

use std::collections::BTreeMap;
use std::fmt;

use lawsynth_core::Identifier;
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory, simulate};
use lawsynth_world::World;

pub use render::{
    format_number, python_number, render_c_expression, render_continuous_law, render_discrete_law,
    render_expression, render_latex_expression, render_latex_law, render_matlab_expression,
    render_python_expression,
};

mod compgraph;
pub use compgraph::{
    ComputationGraph, GraphNode, GraphOp, build_computation_graph, evaluate_graph,
    render_computation_graph_json,
};
pub use svg::{
    FitSeries, RegimeSpan, fit_overlay_chart, fit_overlay_chart_themed, line_chart,
    line_chart_themed, phase_portrait, phase_portrait_themed, regime_timeline,
    regime_timeline_themed, residual_strip, residual_strip_themed, series_color,
    uncertainty_band_chart, uncertainty_band_chart_themed,
};

mod comparison;
pub use comparison::{
    ComparisonOptions, render_comparison, render_comparison_with_options,
    render_comparison_with_theme,
};

mod backtest;
pub use backtest::{BacktestHorizonPoint, BacktestOriginRow, BacktestReport, render_backtest};

mod scenarios;
pub use scenarios::{
    ScenarioOutcome, ScenarioReport, render_scenarios, render_scenarios_with_theme,
};

mod model_card;
pub use model_card::{
    BacktestSection, EnsembleSection, FitRow, ModelCard, TermStability, ValidationSection,
    render_model_card,
};

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
    /// Whether to render the qualitative "Dynamics analysis" section (fixed
    /// points & stability, largest Lyapunov exponent, conserved quantities).
    ///
    /// Defaults to `true`. The section is skipped with an honest note when the
    /// world's field is non-autonomous after parameter substitution or has no
    /// states; set this to `false` to omit it entirely (byte-identical to the
    /// pre-feature report for that path).
    pub include_dynamics: bool,
    /// Brand theme applied to the document and its charts.
    ///
    /// Defaults to [`Theme::default`] (brand light), so existing callers that
    /// build `ReportOptions` via `..Default::default()` inherit the brand.
    pub theme: Theme,
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
            include_dynamics: true,
            theme: Theme::default(),
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
///
/// The report is themed by [`ReportOptions::theme`] (brand light by default).
pub fn render_report(world: &World, options: &ReportOptions) -> Result<String, ReportError> {
    let trajectory = simulate_default(world, options)?;
    Ok(html::page(&options.title, world, &trajectory, options))
}

/// Renders a report with an explicit [`Theme`], overriding `options.theme`.
///
/// Convenience wrapper for callers that hold a base [`ReportOptions`] and want
/// to swap only the theme without rebuilding the struct.
pub fn render_report_with_theme(
    world: &World,
    options: &ReportOptions,
    theme: Theme,
) -> Result<String, ReportError> {
    let themed = ReportOptions { theme, ..options.clone() };
    render_report(world, &themed)
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
    fn default_report_carries_brand_theme() {
        let html = render_report(&decay_world(), &ReportOptions::default()).unwrap();
        // Brand tokens in the inline stylesheet and charts.
        assert!(html.contains("#18201d"), "brand ink absent"); // ink
        assert!(html.contains("#f3f0e8"), "brand paper absent"); // paper
        assert!(html.contains("#b54b2a"), "brand accent absent"); // accent (primary series + header)
        // Brand font stacks (no external fonts, stacks only).
        assert!(html.contains("Georgia"), "serif display stack absent");
        assert!(html.contains("Inter, system-ui"), "sans interface stack absent");
        assert!(html.contains("ui-monospace"), "mono stack absent");
        // Not the old ad-hoc blue palette.
        assert!(!html.contains("#2563eb"), "legacy blue accent leaked");
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

    /// Damped linear oscillator: ẋ = y, ẏ = -x - c·y with c = 0.3. A stable
    /// spiral at the origin; dissipative (Σλ = -c) with no quadratic invariant.
    fn damped_oscillator_world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [Parameter::new(id("c"), 0.3)],
            [
                ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
                ContinuousLaw::new(
                    id("y"),
                    Expr::difference(
                        Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
                        Expr::product(Expr::symbol(id("c")), Expr::symbol(id("y"))),
                    ),
                ),
            ],
        )
        .unwrap()
    }

    /// Undamped harmonic oscillator: ẋ = y, ẏ = -x. A center at the origin
    /// (inconclusive) with conserved energy x² + y² and ≈ zero exponents.
    fn harmonic_oscillator_world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            Vec::<Parameter>::new(),
            [
                ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
                ContinuousLaw::new(
                    id("y"),
                    Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn damped_world_reports_stable_spiral_and_dissipative_verdict() {
        let html = render_report(&damped_oscillator_world(), &ReportOptions::default()).unwrap();
        assert!(html.contains("Dynamics analysis"));
        assert!(html.contains("Fixed points"));
        // A stable spiral at the origin.
        assert!(html.contains("stable spiral"), "expected a stable spiral verdict");
        // Dissipative Lyapunov verdict (largest exponent < 0).
        assert!(html.contains("dissipative"), "expected a dissipative chaos verdict");
        // No conserved quadratic for a dissipative system.
        assert!(
            html.contains("No conserved quantity found in the polynomial basis"),
            "expected an honest no-invariant note"
        );
    }

    #[test]
    fn harmonic_world_reports_center_energy_and_neutral_verdict() {
        let html = render_report(&harmonic_oscillator_world(), &ReportOptions::default()).unwrap();
        assert!(html.contains("Dynamics analysis"));
        // A center at the origin, flagged inconclusive.
        assert!(html.contains("center"), "expected a center classification");
        assert!(html.contains("inconclusive"), "expected the inconclusive note");
        // Conserved energy H = x^2 + y^2.
        assert!(html.contains("x^2 + y^2"), "expected the conserved energy x^2 + y^2");
        // Neutral / conservative chaos verdict (largest exponent ≈ 0).
        assert!(html.contains("neutral"), "expected a neutral chaos verdict");
    }

    #[test]
    fn dynamics_section_is_deterministic() {
        let world = harmonic_oscillator_world();
        let options = ReportOptions::default();
        assert_eq!(
            render_report(&world, &options).unwrap(),
            render_report(&world, &options).unwrap()
        );
    }

    #[test]
    fn no_analysis_omits_section_and_pins_pre_feature_output() {
        let world = damped_oscillator_world();
        let with = ReportOptions::default();
        let without = ReportOptions { include_dynamics: false, ..ReportOptions::default() };

        let html_on = render_report(&world, &with).unwrap();
        let html_off = render_report(&world, &without).unwrap();

        // The gated section is present with the flag on, absent with it off.
        assert!(html_on.contains("Dynamics analysis"));
        assert!(!html_off.contains("Dynamics analysis"));

        // Byte-stable: the no-analysis output is the with-analysis output minus
        // exactly the appended dynamics section (nothing else changes), which is
        // the pre-feature document for that path.
        let marker = "  <section>\n    <h2>Dynamics analysis</h2>";
        let start = html_on.find(marker).expect("dynamics section present");
        let main_close = html_on[start..].find("</main>").expect("closing main tag") + start;
        let spliced = format!("{}{}", &html_on[..start], &html_on[main_close..]);
        assert_eq!(
            spliced, html_off,
            "no-analysis output must be byte-identical minus the section"
        );
    }

    #[test]
    fn non_autonomous_world_skips_section_honestly() {
        // A field that references a non-state symbol after substitution is
        // non-autonomous: dx/dt = u * x with `u` an exogenous input. The section
        // skips with an honest note rather than fabricating a verdict. Tested at
        // the section level with a supplied trajectory because such a world
        // cannot be forward-simulated (no value for `u`), so `render_report`
        // never reaches the analysis stage.
        let world = World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("u"), VariableRole::Exogenous),
            ],
            Vec::<Parameter>::new(),
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("u")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let trajectory = Trajectory {
            time: vec![0.0, 1.0],
            values: [(id("x"), vec![1.0, 0.5])].into_iter().collect(),
        };
        let mut body = String::new();
        analysis::dynamics_analysis_section(&mut body, &world, &trajectory, 1.0, &Theme::default());
        assert!(body.contains("Dynamics analysis"));
        assert!(body.contains("non-autonomous"), "expected an honest non-autonomous skip note");
        // The skipped section must not fabricate any verdict.
        assert!(!body.contains("Fixed points"));
    }

    #[test]
    fn stateless_world_skips_section_honestly() {
        // Zero states: nothing to analyze. The section says so instead of erroring.
        let empty = World::new(
            Vec::<Variable>::new(),
            Vec::<Parameter>::new(),
            Vec::<ContinuousLaw>::new(),
        )
        .unwrap();
        let trajectory = Trajectory { time: vec![0.0], values: BTreeMap::new() };
        let mut body = String::new();
        analysis::dynamics_analysis_section(&mut body, &empty, &trajectory, 1.0, &Theme::default());
        assert!(body.contains("no state variables"), "expected an honest stateless skip note");
    }
}
