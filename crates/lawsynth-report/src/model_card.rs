//! Standardized, self-contained **model card** for a recovered law system.
//!
//! A model card is a governance document (P9): it bundles the recovered laws,
//! the assumptions the recovery is contingent on, in-window fit quality,
//! **out-of-sample** skill (a holdout validation and a rolling-origin backtest),
//! ensemble term-stability (robust vs unstable terms), and an explicit
//! "known limitations / not validated" section. It renders as branded,
//! dependency-free HTML reusing the report crate's [`Theme`](crate::Theme) and
//! inline-SVG charting.
//!
//! Honesty is a hard requirement: a section whose input was never measured is
//! rendered as an explicit **"Not measured"** placeholder, never fabricated.

use std::fmt::Write;

use lawsynth_world::World;

use crate::html::{document, escape};
use crate::render::{format_number, render_continuous_law};
use crate::svg::line_chart_themed;
use crate::{ReportError, ReportOptions, simulate_default};

/// Renders an optional measured number, marking an unmeasured field absent.
fn optional_number(value: Option<f64>) -> String {
    match value {
        Some(number) => format_number(number),
        None => "&mdash;".to_owned(),
    }
}

/// A per-state accuracy row (R² / RMSE). `None` fields are unmeasured, not zero.
#[derive(Clone, Debug, PartialEq)]
pub struct FitRow {
    /// State variable the scores describe.
    pub state: String,
    /// Coefficient of determination, if measured.
    pub r_squared: Option<f64>,
    /// Root-mean-square error, if measured.
    pub rmse: Option<f64>,
}

/// Out-of-sample **holdout** validation: re-fit on a leading window, scored on
/// the held-out tail. Distinct from in-window fit — it estimates generalization.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidationSection {
    /// Fraction of the series held out for scoring (0..1).
    pub holdout_fraction: f64,
    /// Number of leading samples used to re-fit.
    pub train_samples: usize,
    /// Number of held-out samples scored.
    pub test_samples: usize,
    /// Per-state out-of-sample accuracy.
    pub per_state: Vec<FitRow>,
    /// Mean R² across states, if measurable.
    pub mean_r_squared: Option<f64>,
    /// Plain-language read on generalization.
    pub verdict: String,
}

/// Rolling-origin (walk-forward) **backtest** skill and its horizon decay.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestSection {
    /// Number of forecast origins evaluated.
    pub origins: usize,
    /// Forecast horizon, in observation steps.
    pub horizon: usize,
    /// Per-state aggregate accuracy over all origins.
    pub per_state: Vec<FitRow>,
    /// Mean R² across states, if measurable.
    pub mean_r_squared: Option<f64>,
    /// Error growth from the first lead to the last (multiplier), if measurable.
    pub decay: Option<f64>,
    /// Plain-language read on forecasting skill.
    pub verdict: String,
}

/// One ensemble term's cross-member stability (robust vs unstable).
#[derive(Clone, Debug, PartialEq)]
pub struct TermStability {
    /// Law target the term belongs to.
    pub target: String,
    /// Human-readable feature (e.g. `x`, `x·y`).
    pub feature: String,
    /// Share of members that selected the term (0..1).
    pub selection_frequency: f64,
    /// Mean coefficient across selecting members.
    pub mean: f64,
    /// Coefficient spread across selecting members.
    pub std: f64,
    /// Whether the term is robust (frequently selected, tight spread).
    pub robust: bool,
}

/// Ensemble term-stability section: robust terms separated from unstable ones.
#[derive(Clone, Debug, PartialEq)]
pub struct EnsembleSection {
    /// Number of ensemble members that discovered successfully.
    pub members: usize,
    /// Per-term stability rows, in the ensemble's canonical order.
    pub terms: Vec<TermStability>,
}

/// The assembled model card: every governance section, each optional.
///
/// An absent (`None`) section is rendered as an explicit "Not measured"
/// placeholder so the card never overstates what was actually evaluated.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModelCard {
    /// Document title.
    pub title: String,
    /// Optional one-line provenance subtitle.
    pub subtitle: Option<String>,
    /// Assumptions the recovery is contingent on.
    pub assumptions: Vec<String>,
    /// In-window fit quality (per state), if measured.
    pub fit: Option<Vec<FitRow>>,
    /// Out-of-sample holdout validation, if measured.
    pub validation: Option<ValidationSection>,
    /// Rolling-origin backtest, if measured.
    pub backtest: Option<BacktestSection>,
    /// Ensemble term stability, if measured.
    pub ensemble: Option<EnsembleSection>,
    /// Explicit known limitations / not-validated caveats.
    pub limitations: Vec<String>,
    /// Optional lineage summary (ordered key/value provenance links).
    pub lineage: Vec<(String, String)>,
}

/// Renders a complete, self-contained HTML model card for `world`.
///
/// The recovered laws, variables and a default forward trajectory come from
/// `world` + `options`; every governance section comes from `card`. The result
/// is a single branded HTML document with no external assets.
pub fn render_model_card(
    world: &World,
    card: &ModelCard,
    options: &ReportOptions,
) -> Result<String, ReportError> {
    let trajectory = simulate_default(world, options)?;
    let theme = &options.theme;
    let title = if card.title.is_empty() { &options.title } else { &card.title };

    let mut body = String::new();
    header_section(&mut body, title, card.subtitle.as_deref(), world);
    laws_section(&mut body, world);
    assumptions_section(&mut body, &card.assumptions);
    fit_section(&mut body, card.fit.as_deref());
    validation_section(&mut body, card.validation.as_ref());
    backtest_section(&mut body, card.backtest.as_ref());
    ensemble_section(&mut body, card.ensemble.as_ref());
    limitations_section(&mut body, &card.limitations);

    // Trajectory context chart (default forward simulation).
    let series: Vec<(String, Vec<f64>)> = world
        .state_ids()
        .filter_map(|id| {
            trajectory.values.get(id).map(|values| (id.as_str().to_owned(), values.clone()))
        })
        .collect();
    body.push_str("  <section>\n    <h2>Default forward trajectory</h2>\n");
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&line_chart_themed(&trajectory.time, &series, 720.0, 320.0, theme));
    body.push_str("    </div>\n  </section>\n");

    lineage_section(&mut body, &card.lineage);

    Ok(document(title, &body, theme))
}

fn header_section(body: &mut String, title: &str, subtitle: Option<&str>, world: &World) {
    let _ = write!(
        body,
        "  <header>\n    <h1>{}</h1>\n    <p class=\"subtitle\">Model card &middot; {} state variable(s) &middot; {} law(s)</p>\n",
        escape(title),
        world.state_ids().count(),
        world.laws().len()
    );
    if let Some(text) = subtitle {
        let _ = writeln!(body, "    <p class=\"muted\">{}</p>", escape(text));
    }
    body.push_str("  </header>\n");
}

fn laws_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Recovered law system</h2>\n");
    body.push_str("    <div class=\"equations\">\n");
    for (target, law) in world.laws() {
        let _ = writeln!(
            body,
            "      <div class=\"equation\">{}</div>",
            escape(&render_continuous_law(target.as_str(), &law.expression))
        );
    }
    body.push_str("    </div>\n  </section>\n");
}

fn assumptions_section(body: &mut String, assumptions: &[String]) {
    body.push_str("  <section>\n    <h2>Assumptions this model is contingent on</h2>\n");
    if assumptions.is_empty() {
        body.push_str("    <p class=\"muted\">Not measured &mdash; no assumptions recorded.</p>\n  </section>\n");
        return;
    }
    body.push_str("    <ul>\n");
    for item in assumptions {
        let _ = writeln!(body, "      <li>{}</li>", escape(item));
    }
    body.push_str("    </ul>\n  </section>\n");
}

/// Writes a "Not measured" placeholder for an absent required section.
fn not_measured(body: &mut String, heading: &str, why: &str) {
    let _ = write!(body, "  <section>\n    <h2>{}</h2>\n", escape(heading));
    let _ = writeln!(body, "    <p class=\"muted\">Not measured &mdash; {}.</p>", escape(why));
    body.push_str("  </section>\n");
}

fn fit_rows_table(body: &mut String, rows: &[FitRow]) {
    body.push_str("    <table>\n      <thead><tr><th>State</th><th>R&sup2;</th><th>RMSE</th></tr></thead>\n      <tbody>\n");
    for row in rows {
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td></tr>",
            escape(&row.state),
            optional_number(row.r_squared),
            optional_number(row.rmse)
        );
    }
    body.push_str("      </tbody>\n    </table>\n");
}

fn fit_section(body: &mut String, fit: Option<&[FitRow]>) {
    let Some(rows) = fit else {
        not_measured(body, "Fit quality (in-window)", "no fit scores were provided");
        return;
    };
    body.push_str("  <section>\n    <h2>Fit quality (in-window)</h2>\n");
    body.push_str("    <p class=\"muted\">Forward simulation from the first observation, scored against the training window. In-window fit is necessary but not sufficient &mdash; see out-of-sample sections below.</p>\n");
    fit_rows_table(body, rows);
    body.push_str("  </section>\n");
}

fn validation_section(body: &mut String, section: Option<&ValidationSection>) {
    let Some(section) = section else {
        not_measured(
            body,
            "Out-of-sample skill \u{2014} holdout validation",
            "no holdout validation was run",
        );
        return;
    };
    body.push_str("  <section>\n    <h2>Out-of-sample skill &mdash; holdout validation</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Model re-fit on the leading {} sample(s) and scored on the held-out final {} sample(s) ({}% holdout). Mean R&sup2; {}. <b>{}</b></p>",
        section.train_samples,
        section.test_samples,
        format_number(section.holdout_fraction * 100.0),
        optional_number(section.mean_r_squared),
        escape(&section.verdict)
    );
    fit_rows_table(body, &section.per_state);
    body.push_str("  </section>\n");
}

fn backtest_section(body: &mut String, section: Option<&BacktestSection>) {
    let Some(section) = section else {
        not_measured(
            body,
            "Out-of-sample skill \u{2014} rolling-origin backtest",
            "no backtest was run",
        );
        return;
    };
    body.push_str(
        "  <section>\n    <h2>Out-of-sample skill &mdash; rolling-origin backtest</h2>\n",
    );
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Walk-forward evaluation from {} origin(s), horizon {} step(s). Mean R&sup2; {}; error grows {}&times; from the first lead to the last. <b>{}</b></p>",
        section.origins,
        section.horizon,
        optional_number(section.mean_r_squared),
        optional_number(section.decay),
        escape(&section.verdict)
    );
    fit_rows_table(body, &section.per_state);
    body.push_str("  </section>\n");
}

fn ensemble_section(body: &mut String, section: Option<&EnsembleSection>) {
    let Some(section) = section else {
        not_measured(body, "Ensemble term stability", "no ensemble was run");
        return;
    };
    body.push_str("  <section>\n    <h2>Ensemble term stability</h2>\n");
    let robust = section.terms.iter().filter(|term| term.robust).count();
    let _ = writeln!(
        body,
        "    <p class=\"muted\">{} bootstrap member(s); {} of {} observed term(s) are robust (frequently selected with a tight coefficient spread). Terms flagged <span class=\"removed\">unstable</span> should not be trusted as structure.</p>",
        section.members,
        robust,
        section.terms.len()
    );
    body.push_str("    <table>\n      <thead><tr><th>Target</th><th>Term</th><th>Selection</th><th>Mean</th><th>Std</th><th>Stability</th></tr></thead>\n      <tbody>\n");
    for term in &section.terms {
        let badge = if term.robust {
            "<span class=\"added\">robust</span>"
        } else if term.selection_frequency < 0.6 {
            "<span class=\"removed\">unstable</span>"
        } else {
            "<span class=\"changed\">borderline</span>"
        };
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}%</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{}</td></tr>",
            escape(&term.target),
            escape(&term.feature),
            format_number(term.selection_frequency * 100.0),
            format_number(term.mean),
            format_number(term.std),
            badge
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn limitations_section(body: &mut String, limitations: &[String]) {
    body.push_str("  <section>\n    <h2>Known limitations / not validated</h2>\n");
    if limitations.is_empty() {
        body.push_str("    <p class=\"muted\">Not measured &mdash; no limitations recorded. Absence of recorded limitations is not evidence of none.</p>\n  </section>\n");
        return;
    }
    body.push_str("    <ul>\n");
    for item in limitations {
        let _ = writeln!(body, "      <li>{}</li>", escape(item));
    }
    body.push_str("    </ul>\n  </section>\n");
}

fn lineage_section(body: &mut String, lineage: &[(String, String)]) {
    if lineage.is_empty() {
        return;
    }
    body.push_str("  <section>\n    <h2>Lineage</h2>\n");
    body.push_str(
        "    <table>\n      <thead><tr><th>Link</th><th>Digest</th></tr></thead>\n      <tbody>\n",
    );
    for (key, value) in lineage {
        let _ = writeln!(
            body,
            "        <tr><td>{}</td><td class=\"mono\">{}</td></tr>",
            escape(key),
            escape(value)
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
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

    fn full_card() -> ModelCard {
        ModelCard {
            title: "Model card \u{2014} decay".to_owned(),
            subtitle: Some("world abcd1234".to_owned()),
            assumptions: vec!["Continuous-time first-order dynamics.".to_owned()],
            fit: Some(vec![FitRow {
                state: "x".to_owned(),
                r_squared: Some(0.999),
                rmse: Some(0.001),
            }]),
            validation: Some(ValidationSection {
                holdout_fraction: 0.25,
                train_samples: 30,
                test_samples: 10,
                per_state: vec![FitRow {
                    state: "x".to_owned(),
                    r_squared: Some(0.98),
                    rmse: Some(0.01),
                }],
                mean_r_squared: Some(0.98),
                verdict: "strong generalization".to_owned(),
            }),
            backtest: Some(BacktestSection {
                origins: 4,
                horizon: 8,
                per_state: vec![FitRow {
                    state: "x".to_owned(),
                    r_squared: Some(0.97),
                    rmse: Some(0.02),
                }],
                mean_r_squared: Some(0.97),
                decay: Some(1.4),
                verdict: "strong forecasting skill".to_owned(),
            }),
            ensemble: Some(EnsembleSection {
                members: 8,
                terms: vec![
                    TermStability {
                        target: "x".to_owned(),
                        feature: "x".to_owned(),
                        selection_frequency: 1.0,
                        mean: -1.0,
                        std: 0.001,
                        robust: true,
                    },
                    TermStability {
                        target: "x".to_owned(),
                        feature: "x^2".to_owned(),
                        selection_frequency: 0.2,
                        mean: 0.01,
                        std: 0.05,
                        robust: false,
                    },
                ],
            }),
            limitations: vec![
                "Extrapolation beyond the observed window is not validated.".to_owned(),
            ],
            lineage: vec![("world".to_owned(), "abcd1234".to_owned())],
        }
    }

    #[test]
    fn renders_a_self_contained_branded_document() {
        let html =
            render_model_card(&decay_world(), &full_card(), &ReportOptions::default()).unwrap();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("Model card"));
        assert!(html.contains("Recovered law system"));
        assert!(html.contains("dx/dt = -x"));
        assert!(html.contains("<svg"));
        // Branded, no external assets or scripts.
        assert!(html.contains("#b54b2a"));
        assert!(!html.contains("src=\"http"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn out_of_sample_sections_are_present_when_measured() {
        let html =
            render_model_card(&decay_world(), &full_card(), &ReportOptions::default()).unwrap();
        assert!(html.contains("holdout validation"));
        assert!(html.contains("rolling-origin backtest"));
        assert!(html.contains("Ensemble term stability"));
        assert!(html.contains("robust"));
        assert!(html.contains("unstable"));
        assert!(html.contains("Known limitations / not validated"));
    }

    #[test]
    fn absent_sections_are_marked_not_measured_never_fabricated() {
        let card = ModelCard { title: "sparse".to_owned(), ..ModelCard::default() };
        let html = render_model_card(&decay_world(), &card, &ReportOptions::default()).unwrap();
        // Every out-of-sample section is still present, explicitly marked absent.
        assert!(html.contains("Not measured"));
        assert!(html.contains("holdout validation"));
        assert!(html.contains("rolling-origin backtest"));
        // No invented numbers leaked into an unmeasured card.
        assert!(!html.contains("R&sup2; 0."));
    }

    #[test]
    fn unmeasured_fit_fields_render_as_absent_dashes() {
        let card = ModelCard {
            title: "partial".to_owned(),
            fit: Some(vec![FitRow { state: "x".to_owned(), r_squared: Some(0.9), rmse: None }]),
            ..ModelCard::default()
        };
        let html = render_model_card(&decay_world(), &card, &ReportOptions::default()).unwrap();
        assert!(html.contains("&mdash;"), "absent RMSE must render as a dash");
    }

    #[test]
    fn is_deterministic() {
        let world = decay_world();
        let card = full_card();
        let options = ReportOptions::default();
        assert_eq!(
            render_model_card(&world, &card, &options).unwrap(),
            render_model_card(&world, &card, &options).unwrap()
        );
    }
}
