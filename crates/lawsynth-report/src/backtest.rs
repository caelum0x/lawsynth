//! Rolling-origin backtest HTML report.
//!
//! [`render_backtest`] produces a single, dependency-free HTML file that shows
//! how a world's forecast skill decays with horizon (a chart of mean error at
//! h=1,2,...,H) alongside a per-origin skill table. It reuses the report crate's
//! themed styling and inline SVG line chart so the artifact matches the rest of
//! the product surface.

use std::fmt::Write;

use crate::html::{document, escape};
use crate::render::format_number;
use crate::svg::line_chart_themed;
use crate::theme::Theme;

/// Aggregated error at one forecast horizon, pooled across every origin.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestHorizonPoint {
    /// Forecast step ahead of the origin (1-based).
    pub horizon: usize,
    /// Mean absolute error pooled across origins and states at this horizon.
    pub mean_abs_error: f64,
    /// Root-mean-square error pooled across origins and states at this horizon.
    pub rmse: f64,
}

/// One origin's aggregate skill across the horizon it was scored over.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestOriginRow {
    /// Zero-based index of the origin within the observation series.
    pub origin_index: usize,
    /// Time coordinate of the origin.
    pub origin_time: f64,
    /// Number of horizon steps scored from this origin.
    pub steps: usize,
    /// Mean coefficient of determination across states (None if all constant).
    pub mean_r2: Option<f64>,
    /// Mean skill versus a persistence baseline across states.
    pub mean_skill: Option<f64>,
    /// Mean RMSE across states from this origin.
    pub mean_rmse: f64,
}

/// Everything needed to render a rolling-origin backtest report.
#[derive(Clone, Debug, PartialEq)]
pub struct BacktestReport {
    /// Document title shown in the header.
    pub title: String,
    /// Label for the world under test (typically the bundle path).
    pub bundle_label: String,
    /// Label for the observation source (typically the data path).
    pub data_label: String,
    /// Number of forecast origins evaluated.
    pub origins: usize,
    /// Maximum forecast horizon (steps) scored from each origin.
    pub horizon: usize,
    /// Per-horizon error decay, ordered by horizon.
    pub decay: Vec<BacktestHorizonPoint>,
    /// Per-origin skill rows, ordered by origin index.
    pub per_origin: Vec<BacktestOriginRow>,
    /// Aggregate trust verdict.
    pub verdict: String,
    /// Brand theme applied to the document and its charts.
    pub theme: Theme,
}

impl Default for BacktestReport {
    fn default() -> Self {
        Self {
            title: "LawSynth Backtest".to_owned(),
            bundle_label: String::new(),
            data_label: String::new(),
            origins: 0,
            horizon: 0,
            decay: Vec::new(),
            per_origin: Vec::new(),
            verdict: String::new(),
            theme: Theme::default(),
        }
    }
}

/// Renders a self-contained rolling-origin backtest report.
pub fn render_backtest(report: &BacktestReport) -> String {
    let theme = &report.theme;
    let mut body = String::new();

    let _ = write!(
        body,
        "  <header>\n    <h1>{}</h1>\n    <p class=\"subtitle\">Rolling-origin backtest &middot; {} origin(s) &middot; horizon {} step(s)</p>\n  </header>\n",
        escape(&report.title),
        report.origins,
        report.horizon
    );

    // Verdict + provenance.
    body.push_str("  <section>\n    <h2>Verdict</h2>\n");
    let _ = writeln!(body, "    <p class=\"mono\">{}</p>", escape(&report.verdict));
    let _ = writeln!(
        body,
        "    <p class=\"muted\">world &middot; {}<br>data &middot; {}</p>",
        escape(&report.bundle_label),
        escape(&report.data_label)
    );
    body.push_str("  </section>\n");

    // Skill-vs-horizon decay chart.
    body.push_str("  <section>\n    <h2>Skill decay with horizon</h2>\n");
    if report.decay.is_empty() {
        body.push_str("    <p class=\"muted\">No horizon data to chart.</p>\n");
    } else {
        let horizons: Vec<f64> = report.decay.iter().map(|point| point.horizon as f64).collect();
        let series = vec![
            (
                "mean |error|".to_owned(),
                report.decay.iter().map(|point| point.mean_abs_error).collect::<Vec<_>>(),
            ),
            ("rmse".to_owned(), report.decay.iter().map(|point| point.rmse).collect::<Vec<_>>()),
        ];
        body.push_str("    <div class=\"chart\">\n");
        body.push_str(&line_chart_themed(&horizons, &series, 720.0, 300.0, theme));
        body.push_str("    </div>\n");
        body.push_str("    <table>\n      <thead><tr><th>Horizon</th><th>Mean |error|</th><th>RMSE</th></tr></thead>\n      <tbody>\n");
        for point in &report.decay {
            let _ = writeln!(
                body,
                "        <tr><td class=\"mono\">h={}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td></tr>",
                point.horizon,
                escape(&format!("{:.4e}", point.mean_abs_error)),
                escape(&format!("{:.4e}", point.rmse))
            );
        }
        body.push_str("      </tbody>\n    </table>\n");
    }
    body.push_str("  </section>\n");

    // Per-origin skill table.
    body.push_str("  <section>\n    <h2>Per-origin skill</h2>\n");
    body.push_str("    <table>\n      <thead><tr><th>Origin</th><th>t</th><th>Steps</th><th>R2</th><th>Skill vs persist</th><th>Mean RMSE</th></tr></thead>\n      <tbody>\n");
    for row in &report.per_origin {
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">#{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td class=\"mono\">{}</td></tr>",
            row.origin_index,
            escape(&format_number(row.origin_time)),
            row.steps,
            escape(
                &row.mean_r2.map(|value| format!("{value:.4}")).unwrap_or_else(|| "n/a".to_owned())
            ),
            escape(
                &row.mean_skill
                    .map(|value| format!("{value:.4}"))
                    .unwrap_or_else(|| "n/a".to_owned())
            ),
            escape(&format!("{:.4e}", row.mean_rmse))
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");

    document(&report.title, &body, theme)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BacktestReport {
        BacktestReport {
            title: "Backtest: model.lsworld".to_owned(),
            bundle_label: "model.lsworld".to_owned(),
            data_label: "obs.csv".to_owned(),
            origins: 3,
            horizon: 4,
            decay: vec![
                BacktestHorizonPoint { horizon: 1, mean_abs_error: 0.01, rmse: 0.02 },
                BacktestHorizonPoint { horizon: 2, mean_abs_error: 0.05, rmse: 0.08 },
                BacktestHorizonPoint { horizon: 3, mean_abs_error: 0.12, rmse: 0.20 },
            ],
            per_origin: vec![BacktestOriginRow {
                origin_index: 0,
                origin_time: 0.0,
                steps: 4,
                mean_r2: Some(0.98),
                mean_skill: Some(0.7),
                mean_rmse: 0.03,
            }],
            verdict: "STRONG - forecasts stay accurate across origins".to_owned(),
            theme: Theme::default(),
        }
    }

    #[test]
    fn renders_a_self_contained_document() {
        let html = render_backtest(&sample());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<svg"));
        assert!(html.contains("Skill decay with horizon"));
        assert!(html.contains("Per-origin skill"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("src=\"http"));
    }

    #[test]
    fn is_deterministic() {
        assert_eq!(render_backtest(&sample()), render_backtest(&sample()));
    }
}
