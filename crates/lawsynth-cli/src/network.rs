//! `lawsynth network` — directed coupling-structure discovery from data.
//!
//! Reads a multi-column time-series dataset, treats the named columns as network
//! nodes, and runs the deterministic strong-form discovery of
//! [`lawsynth_network::discover_network`] to recover the **directed coupling
//! graph**: which node drives which. It prints the recovered edge list
//! `j -> i (strength)` — a directed influence of node `j` on node `i`'s dynamics
//! — and, under `--json`, the full boolean adjacency and strength matrices.
//!
//! The recovered graph is **correlational**, not causal: a confounder or common
//! drive can induce a spurious edge, and only couplings the polynomial library can
//! represent and that clear `--edge-threshold` are recovered. Targets come from
//! numerical differentiation, so heavy observation noise degrades recovery as it
//! does for strong-form SINDy.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn};
use lawsynth_network::{NetworkConfig, NetworkModel, discover_network};
use lawsynth_report::format_number;

use crate::analysis::{json_string, parse_identifiers, parse_positive, parse_usize};
use crate::read_numeric_dataset;

/// Help text for `lawsynth network`.
pub fn help() -> String {
    "lawsynth network OBSERVATIONS.{csv,tsv,parquet} --state NAME[,NAME...] \
[--degree D] [--threshold T] [--edge-threshold E] [--time COLUMN] [--json]\n\n\
Discovers the directed coupling graph of a networked system: each named state \
column is a node, its derivative is estimated and sparsely regressed onto a shared \
polynomial library over all nodes, and a surviving cross term x_j in node i's \
equation is reported as a directed edge j -> i. Prints the recovered edge list with \
per-edge strengths. --degree sets the library degree (1 = linear couplings), \
--threshold the per-term sparsity cutoff, and --edge-threshold the minimum \
aggregated strength for an edge. The graph is correlational, not causal. --json \
emits the boolean adjacency and strength matrices."
        .to_owned()
}

/// Runs the `network` command.
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

    let mut states = None;
    let mut time_column = None;
    let mut degree = None;
    let mut threshold = None;
    let mut edge_threshold = None;
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option == "--json" {
            as_json = true;
            index += 1;
            continue;
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--state" => states = Some(parse_identifiers(value)?),
            "--time" => time_column = Some(value.clone()),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            "--threshold" => threshold = Some(parse_positive(value, "--threshold")?),
            "--edge-threshold" => edge_threshold = Some(parse_edge_threshold(value)?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let states = states.ok_or_else(|| "--state NAME[,NAME...] is required".to_owned())?;
    if states.len() < 2 {
        return Err("--state must name at least two nodes to recover a coupling".to_owned());
    }
    let time_column = time_column.unwrap_or_else(|| "time".to_owned());

    let dataset = read_numeric_dataset(input, &time_column)?;
    let dataset = select_nodes(&dataset, &states)?;

    let mut config = NetworkConfig::default();
    if let Some(degree) = degree {
        config.features.polynomial_degree = degree;
    }
    if let Some(threshold) = threshold {
        config.sparse.threshold = threshold;
    }
    if let Some(edge_threshold) = edge_threshold {
        config.edge_threshold = edge_threshold;
    }

    let model = discover_network(&dataset, &config).map_err(|error| error.to_string())?;

    if as_json { Ok(render_json(input, &model)) } else { Ok(render_text(input, &model)) }
}

/// Builds a dataset containing only the requested node columns, preserving the
/// time axis. Every requested state must be a column of the source dataset.
fn select_nodes(dataset: &Dataset, states: &[Identifier]) -> Result<Dataset, String> {
    let mut columns = Vec::with_capacity(states.len());
    for state in states {
        let column = dataset.columns().get(state).ok_or_else(|| {
            format!("dataset has no column '{}' named in --state", state.as_str())
        })?;
        columns.push(NumericColumn::new(state.clone(), column.values.clone()));
    }
    Dataset::new(dataset.time().clone(), columns).map_err(|error| error.to_string())
}

/// Parses the edge threshold, requiring a finite, non-negative value.
fn parse_edge_threshold(value: &str) -> Result<f64, String> {
    let number: f64 =
        value.parse().map_err(|_| format!("invalid number '{value}' for --edge-threshold"))?;
    if !number.is_finite() || number < 0.0 {
        return Err("--edge-threshold must be finite and >= 0".to_owned());
    }
    Ok(number)
}

/// Every directed edge `j -> i` in the model, as `(driver_index, target_index)`
/// pairs, in ascending `(target, driver)` order. Includes self loops.
fn edges(model: &NetworkModel) -> Vec<(usize, usize)> {
    let mut list = Vec::new();
    for i in 0..model.len() {
        for j in 0..model.len() {
            if model.is_edge(i, j) {
                list.push((j, i));
            }
        }
    }
    list
}

/// Human-facing report.
fn render_text(source: &str, model: &NetworkModel) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Network discovery from {source}");
    let nodes: Vec<&str> = model.nodes.iter().map(Identifier::as_str).collect();
    let _ = writeln!(out, "  nodes:   {}", nodes.join(", "));
    let _ = writeln!(out, "  library: {} shared term(s)", model.library_terms.len());
    let edges = edges(model);
    let _ = writeln!(out, "  edges:   {}", edges.len());
    out.push('\n');

    if edges.is_empty() {
        let _ = writeln!(
            out,
            "No coupling edges recovered above the edge threshold. Lower \
--edge-threshold or raise --degree to admit weaker or nonlinear couplings."
        );
        return out;
    }

    let _ = writeln!(out, "Directed edges (driver -> target):");
    for (driver, target) in edges {
        let strength = model.edge_strength(target, driver);
        let marker = if driver == target { "  (self)" } else { "" };
        let _ = writeln!(
            out,
            "  {} -> {} (strength {}){}",
            model.nodes[driver].as_str(),
            model.nodes[target].as_str(),
            format_number(strength),
            marker
        );
    }
    out
}

/// Stable, machine-readable report: nodes, the boolean adjacency, the strength
/// matrix, and the flattened edge list. `adjacency[i][j]` means `j -> i`.
fn render_json(source: &str, model: &NetworkModel) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let nodes: Vec<String> = model.nodes.iter().map(|node| json_string(node.as_str())).collect();
    let _ = writeln!(out, "  \"nodes\": [{}],", nodes.join(", "));

    let adjacency_rows: Vec<String> = model
        .adjacency
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|edge| edge.to_string()).collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    let _ = writeln!(out, "  \"adjacency\": [{}],", adjacency_rows.join(", "));

    let strength_rows: Vec<String> = model
        .strength
        .iter()
        .map(|row| {
            let cells: Vec<String> = row.iter().map(|value| format!("{value:.17e}")).collect();
            format!("[{}]", cells.join(", "))
        })
        .collect();
    let _ = writeln!(out, "  \"strength\": [{}],", strength_rows.join(", "));

    let _ = writeln!(out, "  \"edges\": [");
    let edges = edges(model);
    for (number, (driver, target)) in edges.iter().enumerate() {
        let strength = model.edge_strength(*target, *driver);
        let _ = write!(
            out,
            "    {{\"driver\": {}, \"target\": {}, \"strength\": {:.17e}}}",
            json_string(model.nodes[*driver].as_str()),
            json_string(model.nodes[*target].as_str()),
            strength
        );
        let _ = writeln!(out, "{}", if number + 1 == edges.len() { "" } else { "," });
    }
    let _ = writeln!(out, "  ]");
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_flags() {
        let help = help();
        assert!(help.contains("--state"));
        assert!(help.contains("--edge-threshold"));
        assert!(help.contains("j -> i"));
    }

    #[test]
    fn edge_threshold_must_be_valid() {
        assert!(parse_edge_threshold("0.05").is_ok());
        assert!(parse_edge_threshold("0").is_ok());
        assert!(parse_edge_threshold("-1").is_err());
        assert!(parse_edge_threshold("x").is_err());
    }

    #[test]
    fn requires_at_least_two_nodes() {
        let error =
            run(&["data.csv".to_owned(), "--state".to_owned(), "x".to_owned()]).unwrap_err();
        assert!(error.contains("at least two nodes"), "error: {error}");
    }
}
