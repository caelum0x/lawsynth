//! `lawsynth report` — render a self-contained HTML report from a world.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use lawsynth_bundle::read_world;
use lawsynth_report::{ReportObservations, ReportOptions, render_report};

use crate::args::parse_assignment_text;
use crate::read_numeric_dataset;

/// Help text for `lawsynth report`.
pub fn help() -> String {
    "lawsynth report WORLD.lsworld [--output REPORT.html] [--title TEXT] \
[--start T] [--end T] [--step DT] [--initial NAME=VALUE]... \
[--data OBS.{csv,tsv,parquet}] [--time COLUMN]\n\n\
Renders a single, dependency-free HTML file: rendered law equations, \
variable/parameter tables, and inline SVG trajectory and phase-portrait charts. \
With --data, overlays simulated vs observed samples and a residual strip so you \
can see how well the world fits the measurements."
        .to_owned()
}

/// Runs the `report` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle == "--help" || bundle == "-h" {
        return Ok(help());
    }

    let mut output: Option<String> = None;
    let mut options = ReportOptions::default();
    let mut initial_overrides: BTreeMap<_, _> = BTreeMap::new();
    let mut data: Option<String> = None;
    let mut time_column = "time".to_owned();
    let (mut start_set, mut end_set, mut step_set) = (false, false, false);

    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => output = Some(value.clone()),
            "--title" => options.title = value.clone(),
            "--start" => {
                options.start = parse_f64(value)?;
                start_set = true;
            }
            "--end" => {
                options.end = parse_f64(value)?;
                end_set = true;
            }
            "--step" => {
                options.step = parse_f64(value)?;
                step_set = true;
            }
            "--data" => data = Some(value.clone()),
            "--time" => time_column = value.clone(),
            "--initial" => {
                let (id, number) =
                    parse_assignment_text(value).map_err(|error| error.to_string())?;
                initial_overrides.insert(id, number);
            }
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = read_world(bundle).map_err(|error| error.to_string())?;

    // When observations are supplied, align the default simulation to them and
    // attach them so the report renders a fit overlay + residual strip.
    let mut overlaid = 0usize;
    if let Some(data_path) = &data {
        let dataset = read_numeric_dataset(data_path, &time_column)?;
        let times = dataset.time().values();
        if times.len() >= 2 {
            if !start_set {
                options.start = times[0];
            }
            if !end_set {
                options.end = times[times.len() - 1];
            }
            if !step_set {
                let spacing = times[1] - times[0];
                options.step = if spacing.is_finite() && spacing > 0.0 { spacing } else { 0.1 };
            }
        }
        // Seed initial conditions from the first observed row for matching states.
        let mut columns = BTreeMap::new();
        for state in world.state_ids() {
            if let Some(column) = dataset.columns().get(state) {
                columns.insert(state.clone(), column.values.clone());
                initial_overrides.entry(state.clone()).or_insert_with(|| column.values[0]);
                overlaid += 1;
            }
        }
        options.observations = Some(ReportObservations { time: times.to_vec(), columns });
    }

    options.initial_overrides = initial_overrides;

    let html = render_report(&world, &options).map_err(|error| error.to_string())?;

    let output_path = output.unwrap_or_else(|| default_output_path(bundle));
    fs::write(&output_path, &html)
        .map_err(|error| format!("failed to write {output_path}: {error}"))?;

    let mut summary = format!(
        "wrote report: {output_path} ({} bytes, {} state variable(s))\n",
        html.len(),
        world.state_ids().count()
    );
    if data.is_some() {
        summary.push_str(&format!("overlaid observations for {overlaid} state(s)\n"));
    }
    Ok(summary)
}

fn default_output_path(bundle: &str) -> String {
    let stem = Path::new(bundle).file_stem().and_then(|stem| stem.to_str()).unwrap_or("report");
    format!("{stem}.report.html")
}

fn parse_f64(value: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}
