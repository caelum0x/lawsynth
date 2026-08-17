//! HTML page assembly for the self-contained world report.

use std::fmt::Write;

use lawsynth_expr::Expr;
use lawsynth_sim::Trajectory;
use lawsynth_world::{VariableRole, World};

use crate::render::{format_number, render_continuous_law};
use crate::svg::{
    FitSeries, fit_overlay_chart_themed, line_chart_themed, phase_portrait_themed,
    regime_timeline_themed, residual_strip_themed, uncertainty_band_chart_themed,
};
use crate::theme::{Theme, stylesheet};
use crate::{ReportObservations, ReportOptions, UncertaintyBand};

/// Escapes text for safe inclusion in HTML element content or attributes.
pub(crate) fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// Counts scalar AST nodes as a deterministic complexity cost.
fn complexity(expression: &Expr) -> usize {
    match expression {
        Expr::Constant(_) | Expr::Symbol(_) => 1,
        Expr::Unary { operand, .. } => 1 + complexity(operand),
        Expr::Binary { left, right, .. } => 1 + complexity(left) + complexity(right),
    }
}

fn role_name(role: VariableRole) -> &'static str {
    match role {
        VariableRole::State => "state",
        VariableRole::Control => "control",
        VariableRole::Exogenous => "exogenous",
        VariableRole::Observed => "observed",
        VariableRole::Latent => "latent",
        VariableRole::Derived => "derived",
    }
}

/// Assembles the complete standalone HTML document.
pub fn page(
    title: &str,
    world: &World,
    trajectory: &Trajectory,
    options: &ReportOptions,
) -> String {
    let mut body = String::new();
    let theme = &options.theme;
    let total_complexity: usize =
        world.laws().values().map(|law| complexity(&law.expression)).sum();

    let _ = write!(
        body,
        "  <header>\n    <h1>{}</h1>\n    <p class=\"subtitle\">Executable world &middot; {} state variable(s) &middot; complexity {}</p>\n  </header>\n",
        escape(title),
        world.state_ids().count(),
        total_complexity
    );

    laws_section(&mut body, world);
    variables_section(&mut body, world);
    parameters_section(&mut body, world);
    trajectory_section(&mut body, world, trajectory, theme);
    if let Some(observations) = &options.observations {
        fit_section(&mut body, world, trajectory, observations, theme);
    }
    if let Some(regimes) = &options.regimes {
        regime_section(&mut body, regimes, theme);
    }
    if let Some(bands) = &options.uncertainty {
        uncertainty_section(&mut body, bands, theme);
    }
    phase_section(&mut body, world, trajectory, theme);

    document(title, &body, theme)
}

/// Linearly interpolates `(source_times, source_values)` onto `query_times`.
///
/// Both series are strictly increasing; out-of-range queries clamp to the
/// nearest endpoint. Empty sources yield NaN so downstream rendering skips them.
fn interpolate_onto(source_times: &[f64], source_values: &[f64], query_times: &[f64]) -> Vec<f64> {
    if source_times.is_empty() {
        return vec![f64::NAN; query_times.len()];
    }
    let last = source_times.len() - 1;
    let mut cursor = 0;
    query_times
        .iter()
        .map(|&query| {
            while cursor + 1 < source_times.len() && source_times[cursor + 1] < query {
                cursor += 1;
            }
            if query <= source_times[0] {
                return source_values[0];
            }
            if query >= source_times[last] {
                return source_values[last];
            }
            let left = cursor;
            let right = (cursor + 1).min(last);
            let span = source_times[right] - source_times[left];
            if span <= 0.0 {
                return source_values[left];
            }
            let fraction = (query - source_times[left]) / span;
            source_values[left] + fraction * (source_values[right] - source_values[left])
        })
        .collect()
}

/// Fit overlay (simulated vs observed) and residual strip for observed states.
fn fit_section(
    body: &mut String,
    world: &World,
    trajectory: &Trajectory,
    observations: &ReportObservations,
    theme: &Theme,
) {
    let mut fit_series: Vec<FitSeries> = Vec::new();
    let mut residuals: Vec<(String, Vec<f64>)> = Vec::new();
    for state in world.state_ids() {
        let (Some(simulated), Some(observed)) =
            (trajectory.values.get(state), observations.columns.get(state))
        else {
            continue;
        };
        // Residual = simulated (interpolated onto observation times) - observed.
        let predicted = interpolate_onto(&trajectory.time, simulated, &observations.time);
        let residual: Vec<f64> = predicted
            .iter()
            .zip(observed.iter())
            .map(|(prediction, actual)| prediction - actual)
            .collect();
        fit_series.push(FitSeries {
            label: state.as_str().to_owned(),
            observed: observed.clone(),
            simulated: simulated.clone(),
        });
        residuals.push((state.as_str().to_owned(), residual));
    }
    if fit_series.is_empty() {
        return;
    }
    // Aggregate RMSE across all overlaid states.
    let (mut sum_squared, mut count) = (0.0, 0usize);
    for (_, residual) in &residuals {
        for value in residual {
            if value.is_finite() {
                sum_squared += value * value;
                count += 1;
            }
        }
    }
    let rmse = if count > 0 { (sum_squared / count as f64).sqrt() } else { f64::NAN };

    body.push_str("  <section>\n    <h2>Fit vs observations</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Simulated trajectory (solid) over observed samples (markers) for {} state(s); aggregate RMSE {}.</p>",
        fit_series.len(),
        format_number(rmse)
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&fit_overlay_chart_themed(
        &trajectory.time,
        &observations.time,
        &fit_series,
        720.0,
        340.0,
        theme,
    ));
    body.push_str("    </div>\n");
    body.push_str("    <p class=\"muted\">Residuals (simulated &minus; observed); stems above the line overpredict, below underpredict.</p>\n");
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&residual_strip_themed(&observations.time, &residuals, 720.0, 170.0, theme));
    body.push_str("    </div>\n  </section>\n");
}

/// Regime timeline for a discovery that carries a segmentation.
fn regime_section(body: &mut String, regimes: &[crate::RegimeSpan], theme: &Theme) {
    if regimes.is_empty() {
        return;
    }
    let total = regimes.iter().map(|span| span.end).max().unwrap_or(0);
    body.push_str("  <section>\n    <h2>Regime timeline</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">{} regime(s) detected across {} sample(s); vertical ticks mark change points.</p>",
        regimes.len(),
        total
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&regime_timeline_themed(regimes, total, 720.0, 110.0, theme));
    body.push_str("    </div>\n  </section>\n");
}

/// Per-state uncertainty bands for a discovery that carries an envelope.
fn uncertainty_section(body: &mut String, bands: &[UncertaintyBand], theme: &Theme) {
    let bands: Vec<&UncertaintyBand> = bands.iter().filter(|band| band.time.len() >= 2).collect();
    if bands.is_empty() {
        return;
    }
    body.push_str("  <section>\n    <h2>Uncertainty bands</h2>\n");
    body.push_str(
        "    <p class=\"muted\">Median trajectory with its uncertainty envelope per state.</p>\n",
    );
    for band in bands {
        let _ = writeln!(body, "    <h3 class=\"mono\">{}</h3>", escape(band.state.as_str()));
        body.push_str("    <div class=\"chart\">\n");
        body.push_str(&uncertainty_band_chart_themed(
            &band.time,
            &band.lower,
            &band.median,
            &band.upper,
            band.state.as_str(),
            720.0,
            300.0,
            theme,
        ));
        body.push_str("    </div>\n");
    }
    body.push_str("  </section>\n");
}

fn laws_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Laws</h2>\n");
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

fn variables_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Variables</h2>\n");
    body.push_str("    <table>\n      <thead><tr><th>Name</th><th>Role</th><th>Unit</th></tr></thead>\n      <tbody>\n");
    for variable in world.variables().values() {
        let unit = variable
            .unit
            .as_ref()
            .map(|unit| escape(unit.canonical()))
            .unwrap_or_else(|| "&mdash;".to_owned());
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td>{}</td><td>{unit}</td></tr>",
            escape(variable.id.as_str()),
            role_name(variable.role)
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn parameters_section(body: &mut String, world: &World) {
    body.push_str("  <section>\n    <h2>Parameters</h2>\n");
    if world.parameters().is_empty() {
        body.push_str("    <p class=\"muted\">No free parameters.</p>\n  </section>\n");
        return;
    }
    body.push_str("    <table>\n      <thead><tr><th>Name</th><th>Value</th><th>Unit</th></tr></thead>\n      <tbody>\n");
    for parameter in world.parameters().values() {
        let unit = parameter
            .unit
            .as_ref()
            .map(|unit| escape(unit.canonical()))
            .unwrap_or_else(|| "&mdash;".to_owned());
        let _ = writeln!(
            body,
            "        <tr><td class=\"mono\">{}</td><td class=\"mono\">{}</td><td>{unit}</td></tr>",
            escape(parameter.id.as_str()),
            format_number(parameter.value)
        );
    }
    body.push_str("      </tbody>\n    </table>\n  </section>\n");
}

fn trajectory_section(body: &mut String, world: &World, trajectory: &Trajectory, theme: &Theme) {
    let series: Vec<(String, Vec<f64>)> = world
        .state_ids()
        .filter_map(|id| {
            trajectory.values.get(id).map(|values| (id.as_str().to_owned(), values.clone()))
        })
        .collect();
    body.push_str("  <section>\n    <h2>Simulated trajectory</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Default forward simulation over t &isin; [{}, {}] ({} samples).</p>",
        format_number(trajectory.time.first().copied().unwrap_or(0.0)),
        format_number(trajectory.time.last().copied().unwrap_or(0.0)),
        trajectory.samples()
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&line_chart_themed(&trajectory.time, &series, 720.0, 340.0, theme));
    body.push_str("    </div>\n  </section>\n");
}

fn phase_section(body: &mut String, world: &World, trajectory: &Trajectory, theme: &Theme) {
    let states: Vec<_> = world.state_ids().collect();
    if states.len() < 2 {
        return;
    }
    let (x_id, y_id) = (states[0], states[1]);
    let (Some(x_values), Some(y_values)) =
        (trajectory.values.get(x_id), trajectory.values.get(y_id))
    else {
        return;
    };
    body.push_str("  <section>\n    <h2>Phase portrait</h2>\n");
    let _ = writeln!(
        body,
        "    <p class=\"muted\">Trajectory in <span class=\"mono\">{}</span>&ndash;<span class=\"mono\">{}</span> space. <span style=\"color:{}\">&#9679;</span> start, <span style=\"color:{}\">&#9679;</span> end.</p>",
        escape(x_id.as_str()),
        escape(y_id.as_str()),
        theme.success,
        theme.danger
    );
    body.push_str("    <div class=\"chart\">\n");
    body.push_str(&phase_portrait_themed(
        x_id.as_str(),
        x_values,
        y_id.as_str(),
        y_values,
        420.0,
        420.0,
        theme,
    ));
    body.push_str("    </div>\n  </section>\n");
}

pub(crate) fn document(title: &str, body: &str, theme: &Theme) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\" />\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\" />\n<title>{}</title>\n<style>\n{}\n</style>\n</head>\n<body>\n<main>\n{}</main>\n</body>\n</html>\n",
        escape(title),
        stylesheet(theme),
        body
    )
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

    #[test]
    fn escapes_angle_brackets() {
        assert_eq!(escape("a<b>&\"c"), "a&lt;b&gt;&amp;&quot;c");
    }

    #[test]
    fn page_is_a_complete_document() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 0.5)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let trajectory = Trajectory {
            time: vec![0.0, 1.0],
            values: [(id("x"), vec![1.0, 0.37])].into_iter().collect(),
        };
        let html = page("Test", &world, &trajectory, &ReportOptions::default());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("dx/dt = -x"));
        assert!(html.contains("<svg"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn observations_render_a_fit_section() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 0.5)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let trajectory = Trajectory {
            time: vec![0.0, 1.0, 2.0],
            values: [(id("x"), vec![1.0, 0.37, 0.14])].into_iter().collect(),
        };
        let options = ReportOptions {
            observations: Some(ReportObservations {
                time: vec![0.0, 1.0, 2.0],
                columns: [(id("x"), vec![1.0, 0.40, 0.12])].into_iter().collect(),
            }),
            regimes: Some(vec![
                crate::RegimeSpan { start: 0, end: 1, label: "0.5".to_owned() },
                crate::RegimeSpan { start: 1, end: 3, label: "0.2".to_owned() },
            ]),
            ..ReportOptions::default()
        };
        let html = page("Fit", &world, &trajectory, &options);
        assert!(html.contains("Fit vs observations"));
        assert!(html.contains("Residuals"));
        assert!(html.contains("Regime timeline"));
    }
}
