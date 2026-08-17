//! Multi-scenario decision report: overlaid trajectories + divergence table.
//!
//! [`render_scenarios`] produces a single, dependency-free HTML document that
//! puts a baseline and N named what-if scenarios side by side: one multi-series
//! line chart per state (every scenario overlaid, with a legend) plus a
//! final-state / divergence table. This is the "compare your options and decide"
//! surface. All rendering is deterministic and reuses the shared report styling,
//! expression formatting, and the multi-series [`line_chart`](crate::svg::line_chart).

use std::collections::BTreeMap;
use std::fmt::Write;

use crate::html::{document, escape};
use crate::render::format_number;
use crate::svg::line_chart;

/// One simulated scenario over the report's shared time axis.
#[derive(Clone, Debug, PartialEq)]
pub struct ScenarioOutcome {
    /// Short scenario label (e.g. `baseline`, `shock`).
    pub label: String,
    /// Human-readable description of the interventions that define it.
    pub interventions: String,
    /// Whether this outcome is the implicit baseline (divergence reference).
    pub is_baseline: bool,
    /// Per-state trajectories keyed by state label, aligned to the time axis.
    pub trajectories: BTreeMap<String, Vec<f64>>,
}

impl ScenarioOutcome {
    /// Returns the last finite sample for `state`, or `NaN` if none exists.
    fn final_value(&self, state: &str) -> f64 {
        self.trajectories
            .get(state)
            .and_then(|values| values.iter().rev().find(|value| value.is_finite()).copied())
            .unwrap_or(f64::NAN)
    }
}

/// A complete decision report over one baseline and N named scenarios.
#[derive(Clone, Debug, PartialEq)]
pub struct ScenarioReport {
    /// Document title.
    pub title: String,
    /// Shared time axis for every scenario trajectory.
    pub time: Vec<f64>,
    /// Ordered state labels (columns of the divergence table).
    pub states: Vec<String>,
    /// Scenarios to overlay; the baseline is identified by `is_baseline`.
    pub scenarios: Vec<ScenarioOutcome>,
}

impl ScenarioReport {
    /// Returns the baseline outcome (first `is_baseline`, else the first entry).
    fn baseline(&self) -> Option<&ScenarioOutcome> {
        self.scenarios.iter().find(|outcome| outcome.is_baseline).or_else(|| self.scenarios.first())
    }
}

/// Renders a self-contained multi-scenario decision report as HTML.
pub fn render_scenarios(report: &ScenarioReport) -> String {
    let mut body = String::new();
    let sample_count = report.time.len();

    let _ = write!(
        body,
        "  <header>\n    <h1>{}</h1>\n    <p class=\"subtitle\">{} scenario(s) over t &isin; [{}, {}] &middot; {} sample(s) &middot; {} state variable(s)</p>\n  </header>\n",
        escape(&report.title),
        report.scenarios.len(),
        format_number(report.time.first().copied().unwrap_or(0.0)),
        format_number(report.time.last().copied().unwrap_or(0.0)),
        sample_count,
        report.states.len(),
    );

    divergence_section(&mut body, report);
    for state in &report.states {
        chart_section(&mut body, report, state);
    }

    document(&report.title, &body)
}

/// Final-state and baseline-divergence table, plus the interventions column.
fn divergence_section(body: &mut String, report: &ScenarioReport) {
    let baseline = report.baseline();
    body.push_str("  <section>\n    <h2>Final state &amp; divergence from baseline</h2>\n");
    body.push_str(
        "    <p class=\"muted\">Per scenario: the final value of each state and its divergence (&Delta;) from the baseline outcome.</p>\n",
    );
    body.push_str("    <table>\n      <thead><tr><th>Scenario</th><th>Interventions</th>");
    for state in &report.states {
        let _ = write!(
            body,
            "<th class=\"mono\">{}</th><th class=\"mono\">&Delta;{}</th>",
            escape(state),
            escape(state)
        );
    }
    body.push_str("</tr></thead>\n      <tbody>\n");

    for outcome in &report.scenarios {
        let name_class = if outcome.is_baseline { "muted" } else { "mono" };
        let _ = write!(
            body,
            "        <tr><td class=\"{}\">{}</td><td>{}</td>",
            name_class,
            escape(&outcome.label),
            escape(&outcome.interventions)
        );
        for state in &report.states {
            let final_value = outcome.final_value(state);
            let cell_final = format_number(final_value);
            if outcome.is_baseline {
                let _ = write!(
                    body,
                    "<td class=\"mono\">{cell_final}</td><td class=\"mono muted\">&mdash;</td>"
                );
            } else {
                let base = baseline.map(|b| b.final_value(state)).unwrap_or(f64::NAN);
                let delta = final_value - base;
                let class = divergence_class(delta);
                let _ = write!(
                    body,
                    "<td class=\"mono\">{cell_final}</td><td class=\"mono {class}\">{}</td>",
                    signed(delta)
                );
            }
        }
        body.push_str("</tr>\n");
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
    body.push_str(
        "  <style>.up{color:#059669;font-weight:600}.down{color:#dc2626;font-weight:600}.flat{color:#64748b}</style>\n",
    );
}

/// One overlaid multi-series chart for a single state across all scenarios.
fn chart_section(body: &mut String, report: &ScenarioReport, state: &str) {
    let series: Vec<(String, Vec<f64>)> = report
        .scenarios
        .iter()
        .map(|outcome| {
            let values = outcome.trajectories.get(state).cloned().unwrap_or_default();
            (outcome.label.clone(), values)
        })
        .collect();
    let _ = write!(
        body,
        "  <section>\n    <h2>State <span class=\"mono\">{}</span> &mdash; all scenarios</h2>\n",
        escape(state)
    );
    body.push_str(
        "    <p class=\"muted\">Every scenario's trajectory overlaid; see the legend for the mapping.</p>\n",
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&line_chart(&report.time, &series, 720.0, 340.0));
    body.push_str("    </div>\n  </section>\n");
}

fn divergence_class(delta: f64) -> &'static str {
    if !delta.is_finite() || delta == 0.0 {
        "flat"
    } else if delta > 0.0 {
        "up"
    } else {
        "down"
    }
}

/// Formats a divergence with an explicit sign (`+`/`-`).
fn signed(delta: f64) -> String {
    if delta == 0.0 || !delta.is_finite() {
        return format_number(delta);
    }
    let magnitude = format_number(delta.abs());
    if delta > 0.0 { format!("+{magnitude}") } else { format!("-{magnitude}") }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report() -> ScenarioReport {
        let baseline = ScenarioOutcome {
            label: "baseline".to_owned(),
            interventions: "(baseline)".to_owned(),
            is_baseline: true,
            trajectories: [("x".to_owned(), vec![1.0, 0.5, 0.25])].into_iter().collect(),
        };
        let shock = ScenarioOutcome {
            label: "shock".to_owned(),
            interventions: "k=2@1".to_owned(),
            is_baseline: false,
            trajectories: [("x".to_owned(), vec![1.0, 0.4, 0.1])].into_iter().collect(),
        };
        ScenarioReport {
            title: "decision".to_owned(),
            time: vec![0.0, 1.0, 2.0],
            states: vec!["x".to_owned()],
            scenarios: vec![baseline, shock],
        }
    }

    #[test]
    fn renders_a_self_contained_document_with_overlaid_chart() {
        let html = render_scenarios(&report());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("</html>"));
        // Multi-series overlay: both scenario labels appear in one chart legend.
        assert!(html.contains("<svg"));
        assert!(html.contains(">baseline<"));
        assert!(html.contains(">shock<"));
        assert!(!html.contains("<script"));
    }

    #[test]
    fn table_reports_signed_divergence() {
        let html = render_scenarios(&report());
        assert!(html.contains("divergence"));
        // shock final (0.1) minus baseline final (0.25) = -0.15.
        assert!(html.contains("-0.15"));
    }

    #[test]
    fn is_deterministic() {
        assert_eq!(render_scenarios(&report()), render_scenarios(&report()));
    }
}
