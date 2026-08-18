//! `lawsynth select` — cross-validated hyperparameter selection on a dataset.
//!
//! Loads observations (via the shared numeric-dataset boundary), builds a base
//! [`lawsynth_discovery::DiscoveryConfig`] from the discovery flags
//! (`--state`, `--solver`, `--trig`, `--rational`), then runs the deterministic
//! time-series cross-validation sweep in
//! [`lawsynth_modelselect::sweep_degrees_thresholds`] over the Cartesian product
//! of the `--degrees` and `--thresholds` grids. It prints the full audit table —
//! every candidate's mean and per-fold held-out score, with the winner marked —
//! and reports the selected hyperparameters. `--json` emits the complete
//! candidate score table plus `best_index`.
//!
//! Selection is honest and library-bounded: a candidate whose discovery or
//! simulation fails on a fold is recorded as a fold *failure* (a documented
//! worst-case score), never silently dropped. The dataset must be long enough to
//! split into `folds + 1` contiguous segments; otherwise the command reports why.

use std::fmt::Write as _;

use lawsynth_discovery::{DiscoveryConfig, SparseMethod};
use lawsynth_modelselect::{
    CandidateScore, CvConfig, CvScheme, ScoreMetric, SelectionReport, sweep_degrees_thresholds,
};

use crate::analysis::{json_string, parse_identifiers, parse_number, parse_usize};
use crate::read_numeric_dataset;

/// Default sparsity thresholds swept when `--thresholds` is omitted (the
/// discovery default alone).
const DEFAULT_THRESHOLD: f64 = 0.05;
/// Default number of cross-validation folds.
const DEFAULT_FOLDS: usize = 3;

/// Help text for `lawsynth select`.
pub fn help() -> String {
    "lawsynth select OBSERVATIONS.{csv,tsv,parquet} --state NAME[,NAME...] \
--degrees D[,D...] [--thresholds T[,T...]] [--folds K] [--scheme forward|rolling] \
[--metric r2|rmse] [--time COLUMN] [--solver stlsq|sr3|frols|ssr|trapping] \
[--trig] [--rational] [--json]\n\n\
Chooses discovery hyperparameters by deterministic time-series cross-validation. \
The timeline is cut into K+1 contiguous segments; each fold discovers on its \
training segment, re-simulates the discovered world across the held-out segment, \
and scores predictive fit (R^2 by default, or negated RMSE). Every candidate in \
the --degrees × --thresholds grid is scored; the candidate with the best mean \
held-out score wins, ties broken toward the simpler model. The full audit table \
(mean and per-fold scores, failures, winner) is printed. --json emits the \
complete candidate table plus best_index.\n\n\
Defaults: --thresholds 0.05, --folds 3, --scheme forward, --metric r2, \
--time time. The dataset must split into K+1 segments."
        .to_owned()
}

/// Runs the `select` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(input) = arguments.first() else {
        return Err(help());
    };
    if input.starts_with('-') {
        return Err(help());
    }

    let mut state = None;
    let mut degrees = None;
    let mut thresholds = None;
    let mut folds = DEFAULT_FOLDS;
    let mut scheme = CvScheme::ForwardChaining;
    let mut metric = ScoreMetric::RSquared;
    let mut time_column = "time".to_owned();
    let mut sparse_method = SparseMethod::default();
    let mut include_trigonometric = false;
    let mut include_rational = false;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        match option {
            "--trig" => {
                include_trigonometric = true;
                index += 1;
                continue;
            }
            "--rational" => {
                include_rational = true;
                index += 1;
                continue;
            }
            "--json" => {
                as_json = true;
                index += 1;
                continue;
            }
            _ => {}
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--state" => state = Some(parse_identifiers(value)?),
            "--degrees" => degrees = Some(parse_degree_list(value)?),
            "--thresholds" => thresholds = Some(parse_threshold_list(value)?),
            "--folds" => folds = parse_usize(value, "--folds")?,
            "--time" => time_column = value.clone(),
            "--scheme" => scheme = parse_scheme(value)?,
            "--metric" => metric = parse_metric(value)?,
            "--solver" => sparse_method = parse_solver(value)?,
            _ => return Err(help()),
        }
        index += 2;
    }

    let state = state.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;
    let degrees = degrees.ok_or_else(|| "--degrees D[,D...] is required".to_owned())?;
    let thresholds = thresholds.unwrap_or_else(|| vec![DEFAULT_THRESHOLD]);
    if folds == 0 {
        return Err("--folds must be at least 1".to_owned());
    }

    let dataset = read_numeric_dataset(input, &time_column)?;

    let mut base = DiscoveryConfig::new(state);
    base.sparse_method = sparse_method;
    base.include_trigonometric = include_trigonometric;
    base.include_rational = include_rational;

    let cv = CvConfig::new(folds).with_scheme(scheme).with_metric(metric);
    let report = sweep_degrees_thresholds(&dataset, &base, &degrees, &thresholds, &cv)
        .map_err(|error| error.to_string())?;

    if as_json { Ok(render_json(input, &report)) } else { Ok(render_text(input, &report)) }
}

/// Human-facing report: the engine's audit table plus a one-line summary of the
/// selected hyperparameters, so the winner is unambiguous.
fn render_text(input: &str, report: &SelectionReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Model selection on {input}");
    out.push_str(&report.render_table());
    let best = report.best();
    out.push('\n');
    let _ = writeln!(
        out,
        "Selected: degree={}, threshold={:.4}, solver={}, mean_score={:.6e}{}",
        best.config.polynomial_degree,
        best.config.threshold,
        solver_label(best.config.sparse_method),
        best.mean_score,
        best.active_terms.map(|terms| format!(", active_terms={terms}")).unwrap_or_default(),
    );
    out
}

/// Stable, machine-readable report: the full candidate table plus `best_index`.
/// Floats use the full 17-digit form so the JSON is a faithful, deterministic
/// image of the report.
fn render_json(input: &str, report: &SelectionReport) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"dataset\": {},", json_string(input));
    let _ = writeln!(out, "  \"scheme\": {},", json_string(scheme_token(report.scheme)));
    let _ = writeln!(out, "  \"metric\": {},", json_string(metric_token(report.metric)));
    let _ = writeln!(out, "  \"folds\": {},", report.folds);
    let _ = writeln!(out, "  \"best_index\": {},", report.best_index);
    let _ = writeln!(out, "  \"candidates\": [");
    for (number, candidate) in report.candidates.iter().enumerate() {
        write_candidate_json(&mut out, candidate, number == report.best_index);
        let terminator = if number + 1 == report.candidates.len() { "    }" } else { "    }," };
        let _ = writeln!(out, "{terminator}");
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

/// Writes the body (without the closing brace) of one candidate JSON object.
fn write_candidate_json(out: &mut String, candidate: &CandidateScore, is_best: bool) {
    let _ = writeln!(out, "    {{");
    let _ = writeln!(out, "      \"grid_index\": {},", candidate.grid_index);
    let _ = writeln!(out, "      \"is_best\": {is_best},");
    let _ = writeln!(out, "      \"degree\": {},", candidate.config.polynomial_degree);
    let _ = writeln!(out, "      \"threshold\": {:.17e},", candidate.config.threshold);
    let _ = writeln!(
        out,
        "      \"solver\": {},",
        json_string(solver_label(candidate.config.sparse_method))
    );
    let _ = writeln!(out, "      \"mean_score\": {:.17e},", candidate.mean_score);
    let _ = writeln!(out, "      \"failed_folds\": {},", candidate.failed_folds);
    match candidate.active_terms {
        Some(terms) => {
            let _ = writeln!(out, "      \"active_terms\": {terms},");
        }
        None => out.push_str("      \"active_terms\": null,\n"),
    }
    let _ = writeln!(out, "      \"fold_scores\": [");
    for (number, fold) in candidate.fold_scores.iter().enumerate() {
        let r_squared = fold
            .r_squared
            .map(|value| format!("{value:.17e}"))
            .unwrap_or_else(|| "null".to_owned());
        let rmse =
            fold.rmse.map(|value| format!("{value:.17e}")).unwrap_or_else(|| "null".to_owned());
        let _ = write!(
            out,
            "        {{\"fold\": {}, \"status\": {}, \"score\": {:.17e}, \"r_squared\": {}, \
\"rmse\": {}}}",
            fold.fold_index,
            json_string(fold_status_token(fold.status)),
            fold.score,
            r_squared,
            rmse
        );
        out.push_str(if number + 1 < candidate.fold_scores.len() { ",\n" } else { "\n" });
    }
    let _ = writeln!(out, "      ]");
}

/// The stable JSON token for a fold status.
fn fold_status_token(status: lawsynth_modelselect::FoldStatus) -> &'static str {
    use lawsynth_modelselect::FoldStatus;
    match status {
        FoldStatus::Scored => "scored",
        FoldStatus::DiscoveryFailed => "discovery-failed",
        FoldStatus::SimulationFailed => "simulation-failed",
        FoldStatus::ScoringFailed => "scoring-failed",
    }
}

/// The stable JSON token for a CV scheme.
fn scheme_token(scheme: CvScheme) -> &'static str {
    match scheme {
        CvScheme::ForwardChaining => "forward-chaining",
        CvScheme::RollingBlocks => "rolling-blocks",
    }
}

/// The stable JSON token for a scoring metric.
fn metric_token(metric: ScoreMetric) -> &'static str {
    match metric {
        ScoreMetric::RSquared => "r2",
        ScoreMetric::Rmse => "rmse",
    }
}

/// Stable string label for the sparse solver (mirrors the `discover` command).
fn solver_label(method: SparseMethod) -> &'static str {
    match method {
        SparseMethod::Stlsq => "stlsq",
        SparseMethod::Sr3 => "sr3",
        SparseMethod::Frols => "frols",
        SparseMethod::Ssr => "ssr",
        SparseMethod::Trapping => "trapping",
    }
}

/// Parses `forward` or `rolling` into a [`CvScheme`].
fn parse_scheme(value: &str) -> Result<CvScheme, String> {
    match value {
        "forward" | "forward-chaining" => Ok(CvScheme::ForwardChaining),
        "rolling" | "rolling-blocks" => Ok(CvScheme::RollingBlocks),
        other => Err(format!("--scheme must be 'forward' or 'rolling', got '{other}'")),
    }
}

/// Parses `r2` or `rmse` into a [`ScoreMetric`].
fn parse_metric(value: &str) -> Result<ScoreMetric, String> {
    match value {
        "r2" | "rsquared" => Ok(ScoreMetric::RSquared),
        "rmse" => Ok(ScoreMetric::Rmse),
        other => Err(format!("--metric must be 'r2' or 'rmse', got '{other}'")),
    }
}

/// Parses a sparse-solver keyword (mirrors the `discover` command).
fn parse_solver(value: &str) -> Result<SparseMethod, String> {
    match value {
        "stlsq" => Ok(SparseMethod::Stlsq),
        "sr3" => Ok(SparseMethod::Sr3),
        "frols" => Ok(SparseMethod::Frols),
        "ssr" => Ok(SparseMethod::Ssr),
        "trapping" => Ok(SparseMethod::Trapping),
        _ => Err("--solver must be 'stlsq', 'sr3', 'frols', 'ssr', or 'trapping'".to_owned()),
    }
}

/// Parses a comma-separated list of polynomial degrees (each at least 1).
fn parse_degree_list(value: &str) -> Result<Vec<usize>, String> {
    let degrees = value
        .split(',')
        .map(|item| parse_usize(item.trim(), "--degrees"))
        .collect::<Result<Vec<_>, _>>()?;
    if degrees.is_empty() {
        return Err("expected at least one degree in --degrees".to_owned());
    }
    if degrees.contains(&0) {
        return Err("--degrees values must be at least 1".to_owned());
    }
    Ok(degrees)
}

/// Parses a comma-separated list of non-negative sparsity thresholds.
fn parse_threshold_list(value: &str) -> Result<Vec<f64>, String> {
    let thresholds = value
        .split(',')
        .map(|item| {
            let threshold = parse_number(item.trim(), "--thresholds")?;
            if threshold < 0.0 {
                Err("--thresholds values must be >= 0".to_owned())
            } else {
                Ok(threshold)
            }
        })
        .collect::<Result<Vec<_>, String>>()?;
    if thresholds.is_empty() {
        return Err("expected at least one threshold in --thresholds".to_owned());
    }
    Ok(thresholds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--degrees"));
        assert!(help.contains("--thresholds"));
        assert!(help.contains("--folds"));
    }

    #[test]
    fn parses_degree_and_threshold_lists() {
        assert_eq!(parse_degree_list("1,2,3").unwrap(), vec![1, 2, 3]);
        assert!(parse_degree_list("0,1").unwrap_err().contains("at least 1"));
        assert_eq!(parse_threshold_list("0.05,0.1").unwrap(), vec![0.05, 0.1]);
        assert!(parse_threshold_list("-1").unwrap_err().contains(">= 0"));
    }

    #[test]
    fn parses_scheme_and_metric_keywords() {
        assert_eq!(parse_scheme("forward").unwrap(), CvScheme::ForwardChaining);
        assert_eq!(parse_scheme("rolling").unwrap(), CvScheme::RollingBlocks);
        assert!(parse_scheme("nope").is_err());
        assert_eq!(parse_metric("r2").unwrap(), ScoreMetric::RSquared);
        assert_eq!(parse_metric("rmse").unwrap(), ScoreMetric::Rmse);
        assert!(parse_metric("mae").is_err());
    }
}
