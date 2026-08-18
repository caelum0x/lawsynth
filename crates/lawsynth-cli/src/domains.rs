//! `lawsynth domains` — curated, self-validated domain presets.
//!
//! A [`DomainPreset`] bundles a candidate feature-library shape, an optional
//! structural template prior, unit hints, and a documented reference law with a
//! deterministic RK4 trajectory generator. This command exposes the
//! [`lawsynth_domains`] registry three ways:
//!
//! - `lawsynth domains` — list every preset with a one-line summary.
//! - `lawsynth domains show NAME` — print a preset's reference law and the
//!   discovery configuration it runs under.
//! - `lawsynth domains run NAME [--json]` — the honest round-trip: synthesize the
//!   preset's clean trajectory, run discovery with the preset's own config, and
//!   report how well the discovered law recovers the reference (per-state RMSE
//!   between the discovered and reference right-hand sides).
//!
//! # Honest limits
//!
//! The round-trip runs on **clean synthetic data** generated from a textbook law.
//! A high recovery score here means the preset's search space contains its own
//! reference law -- not that discovery will recover a law from noisy real
//! measurements. Real data needs the noise/smoothing knobs on `discover`.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;
use lawsynth_discovery::discover;
use lawsynth_domains::{DomainPreset, ReferenceLaw, ReferenceTerm, names, preset};
use lawsynth_expr::{Environment, evaluate};
use lawsynth_report::{format_number, render_continuous_law};
use lawsynth_world::World;

/// Recovery is reported as `recovered` when every state's RHS RMSE is at or below
/// this bound on the clean reference trajectory.
const RECOVERY_TOLERANCE: f64 = 1e-3;

/// Help text for `lawsynth domains`.
pub fn help() -> String {
    "lawsynth domains\n  lawsynth domains show NAME\n  lawsynth domains run NAME [--json]\n\n\
Lists the curated domain presets, shows a preset's reference law and discovery \
config, or runs a preset's round-trip recovery (synthesize its clean trajectory, \
discover, and report per-state RHS RMSE). Round-trip is on clean synthetic data: \
it validates the preset's search space, not robustness to real noise."
        .to_owned()
}

/// Runs the `domains` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    match arguments.first().map(String::as_str) {
        None => Ok(render_list()),
        Some("--help" | "-h") => Ok(help()),
        Some("show") => {
            let name =
                arguments.get(1).ok_or_else(|| "usage: lawsynth domains show NAME".to_owned())?;
            let preset = preset(name).map_err(|error| error.to_string())?;
            Ok(render_show(&preset))
        }
        Some("run") => {
            let name =
                arguments.get(1).ok_or_else(|| "usage: lawsynth domains run NAME".to_owned())?;
            let mut as_json = false;
            for extra in &arguments[2..] {
                match extra.as_str() {
                    "--json" => as_json = true,
                    other => return Err(format!("unexpected argument '{other}'\n\n{}", help())),
                }
            }
            let preset = preset(name).map_err(|error| error.to_string())?;
            run_round_trip(&preset, as_json)
        }
        Some(other) => Err(format!("unknown subcommand '{other}'\n\n{}", help())),
    }
}

/// Lists every registered preset in canonical order.
fn render_list() -> String {
    let mut out = String::from("Domain presets (use with `domains show|run <name>`):\n\n");
    for name in names() {
        let preset = preset(name).expect("registered name resolves");
        let _ = writeln!(out, "  {name}");
        let _ = writeln!(out, "    {}", preset.summary());
    }
    out
}

/// Shows a preset's reference law and discovery configuration.
fn render_show(preset: &DomainPreset) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Domain preset: {}", preset.name());
    let _ = writeln!(out, "  {}", preset.summary());
    out.push('\n');

    let variables = preset.state_variables();
    let names: Vec<&str> = variables.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "Reference law (state order: {}):", names.join(", "));
    let reference = preset.reference();
    for variable in variables {
        if let Some(law) = reference.law(variable) {
            let _ = writeln!(
                out,
                "  d/dt {} = {}",
                variable.as_str(),
                render_reference_law(variables, law)
            );
        }
    }
    let _ = writeln!(
        out,
        "  initial: [{}]  dt={}  steps={}  ({} samples)",
        reference
            .initial()
            .iter()
            .map(|value| format_number(*value))
            .collect::<Vec<_>>()
            .join(", "),
        format_number(reference.dt()),
        reference.steps(),
        reference.steps() + 1
    );
    out.push('\n');

    let feature = preset.feature_config();
    let _ = writeln!(out, "Discovery configuration:");
    let _ = writeln!(out, "  polynomial degree:  {}", feature.polynomial_degree);
    let _ = writeln!(out, "  trigonometric:      {}", preset.include_trigonometric());
    let _ = writeln!(out, "  rational:           {}", preset.include_rational());
    let _ = writeln!(
        out,
        "  template prior:     {}",
        if preset.template_prior().is_some() { "yes" } else { "none" }
    );
    if preset.unit_hints().is_empty() {
        let _ = writeln!(out, "  unit hints:         (none)");
    } else {
        let hints: Vec<String> = preset
            .unit_hints()
            .iter()
            .map(|hint| format!("{}={}", hint.variable.as_str(), hint.unit.canonical()))
            .collect();
        let _ = writeln!(out, "  unit hints:         {}", hints.join(", "));
    }
    out
}

/// Runs a preset's round-trip and reports recovery.
fn run_round_trip(preset: &DomainPreset, as_json: bool) -> Result<String, String> {
    let data = preset.reference().trajectory();
    let config = preset.discovery_config();
    let result = discover(&data, &config).map_err(|error| error.to_string())?;
    let candidate = result
        .candidates
        .into_iter()
        .next()
        .ok_or_else(|| "discovery produced no candidates".to_owned())?;

    let recovery = recovery_scores(preset, &data, &candidate.world)?;
    if as_json {
        Ok(render_run_json(preset, &candidate.world, &recovery))
    } else {
        Ok(render_run_text(preset, &candidate.world, &recovery))
    }
}

/// Per-state recovery: the RMSE between the discovered and reference right-hand
/// sides, evaluated at every trajectory sample.
struct StateRecovery {
    state: Identifier,
    rmse: f64,
    discovered_terms: usize,
    reference_terms: usize,
}

/// Evaluates both the reference law and the discovered law over the clean
/// trajectory and returns the per-state RHS RMSE.
fn recovery_scores(
    preset: &DomainPreset,
    data: &Dataset,
    world: &World,
) -> Result<Vec<StateRecovery>, String> {
    let variables = preset.state_variables();
    let columns = data.columns();
    let samples = data.time().values().len();
    let reference = preset.reference();

    let mut scores = Vec::with_capacity(variables.len());
    for state in variables {
        let law = world
            .laws()
            .get(state)
            .ok_or_else(|| format!("discovered world is missing a law for '{}'", state.as_str()))?;
        let reference_law = reference
            .law(state)
            .ok_or_else(|| format!("reference is missing a law for '{}'", state.as_str()))?;

        let mut sum_squares = 0.0_f64;
        for row in 0..samples {
            let point: Vec<f64> = variables.iter().map(|v| columns[v].values[row]).collect();
            let mut environment = Environment::new();
            for (variable, value) in variables.iter().zip(&point) {
                environment.insert(variable.clone(), *value);
            }
            let discovered =
                evaluate(&law.expression, &environment).map_err(|error| error.to_string())?;
            let expected =
                reference.evaluate_law(state, &point).expect("state has a reference law");
            let difference = discovered - expected;
            sum_squares += difference * difference;
        }
        let rmse = (sum_squares / samples as f64).sqrt();
        scores.push(StateRecovery {
            state: state.clone(),
            rmse,
            discovered_terms: count_terms(&law.expression),
            reference_terms: reference_law.active_terms(),
        });
    }
    Ok(scores)
}

/// Counts additive terms in a discovered law's expression tree.
fn count_terms(expression: &lawsynth_expr::Expr) -> usize {
    use lawsynth_expr::{BinaryOperator, Expr};
    match expression {
        Expr::Binary { operator: BinaryOperator::Add | BinaryOperator::Subtract, left, right } => {
            count_terms(left) + count_terms(right)
        }
        _ => 1,
    }
}

/// Whether every state recovered within tolerance.
fn all_recovered(recovery: &[StateRecovery]) -> bool {
    recovery.iter().all(|score| score.rmse <= RECOVERY_TOLERANCE)
}

fn render_run_text(preset: &DomainPreset, world: &World, recovery: &[StateRecovery]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Round-trip recovery for preset '{}'", preset.name());
    let _ = writeln!(out, "  {}", preset.summary());
    out.push('\n');

    let _ = writeln!(out, "Discovered law(s):");
    for (target, law) in world.laws() {
        let _ = writeln!(out, "  {}", render_continuous_law(target.as_str(), &law.expression));
    }
    out.push('\n');

    let _ = writeln!(out, "Recovery vs. reference (clean trajectory):");
    for score in recovery {
        let verdict = if score.rmse <= RECOVERY_TOLERANCE { "recovered" } else { "MISMATCH" };
        let _ = writeln!(
            out,
            "  {:<12} RHS RMSE={}  terms {}/{} (discovered/reference)  -> {verdict}",
            score.state.as_str(),
            format_number(score.rmse),
            score.discovered_terms,
            score.reference_terms
        );
    }
    out.push('\n');
    if all_recovered(recovery) {
        let _ = writeln!(
            out,
            "Recovery: OK (every state within RMSE tolerance {RECOVERY_TOLERANCE:.0e} on clean data)."
        );
    } else {
        let _ = writeln!(
            out,
            "Recovery: incomplete (a state exceeded RMSE tolerance {RECOVERY_TOLERANCE:.0e})."
        );
    }
    let _ = writeln!(
        out,
        "Note: round-trip is on clean synthetic data; it validates the search space, not noise robustness."
    );
    out
}

fn render_run_json(preset: &DomainPreset, world: &World, recovery: &[StateRecovery]) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"preset\": {},", json_string(preset.name()));
    let _ = writeln!(out, "  \"recovered\": {},", all_recovered(recovery));
    let _ = writeln!(out, "  \"tolerance\": {RECOVERY_TOLERANCE:.17e},");
    let _ = writeln!(out, "  \"laws\": [");
    let laws: Vec<(&Identifier, _)> = world.laws().iter().collect();
    for (number, (target, law)) in laws.iter().enumerate() {
        let rendered = render_continuous_law(target.as_str(), &law.expression);
        let terminator = if number + 1 == laws.len() { "" } else { "," };
        let _ = writeln!(out, "    {}{terminator}", json_string(&rendered));
    }
    let _ = writeln!(out, "  ],");
    let _ = writeln!(out, "  \"recovery\": [");
    for (number, score) in recovery.iter().enumerate() {
        let terminator = if number + 1 == recovery.len() { "" } else { "," };
        let _ = writeln!(
            out,
            "    {{\"state\": {}, \"rhs_rmse\": {:.17e}, \"discovered_terms\": {}, \"reference_terms\": {}}}{terminator}",
            json_string(score.state.as_str()),
            score.rmse,
            score.discovered_terms,
            score.reference_terms
        );
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// Renders a reference law `Σ coefficient·∏ varᵢ^expᵢ` as a readable polynomial.
fn render_reference_law(variables: &[Identifier], law: &ReferenceLaw) -> String {
    let terms: Vec<String> = law
        .terms()
        .iter()
        .filter(|term| term.coefficient != 0.0)
        .map(|term| render_reference_term(variables, term))
        .collect();
    if terms.is_empty() { "0".to_owned() } else { terms.join(" + ") }
}

/// Renders one monomial term.
fn render_reference_term(variables: &[Identifier], term: &ReferenceTerm) -> String {
    let mut factors = vec![format_number(term.coefficient)];
    for (variable, &exponent) in variables.iter().zip(&term.exponents) {
        match exponent {
            0 => {}
            1 => factors.push(variable.as_str().to_owned()),
            other => factors.push(format!("{}^{other}", variable.as_str())),
        }
    }
    factors.join("*")
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_every_registered_preset() {
        let listing = render_list();
        for name in names() {
            assert!(listing.contains(name), "listing should mention {name}");
        }
    }

    #[test]
    fn show_renders_reference_and_config() {
        let preset = preset("lotka-volterra").unwrap();
        let text = render_show(&preset);
        assert!(text.contains("Reference law"));
        assert!(text.contains("polynomial degree"));
        assert!(text.contains("d/dt"));
    }

    #[test]
    fn round_trip_recovers_a_preset_law() {
        let output = run(&["run".to_owned(), "lotka-volterra".to_owned()]).unwrap();
        assert!(output.contains("Round-trip recovery"));
        assert!(output.contains("recovered"));
        assert!(output.contains("Recovery: OK"));
    }

    #[test]
    fn round_trip_json_reports_recovered_flag() {
        let output =
            run(&["run".to_owned(), "damped-oscillator".to_owned(), "--json".to_owned()]).unwrap();
        assert!(output.contains("\"recovered\": true"));
        assert!(output.contains("\"rhs_rmse\""));
    }

    #[test]
    fn unknown_preset_is_rejected() {
        let error = run(&["show".to_owned(), "navier-stokes".to_owned()]).unwrap_err();
        assert!(error.contains("navier-stokes"));
    }
}
