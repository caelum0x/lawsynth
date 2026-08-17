//! `lawsynth report` — render a self-contained HTML report from a world.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use lawsynth_bundle::read_world;
use lawsynth_report::{ReportOptions, render_report};

use crate::args::parse_assignment_text;

/// Help text for `lawsynth report`.
pub fn help() -> String {
    "lawsynth report WORLD.lsworld [--output REPORT.html] [--title TEXT] \
[--start T] [--end T] [--step DT] [--initial NAME=VALUE]...\n\n\
Renders a single, dependency-free HTML file: rendered law equations, \
variable/parameter tables, and inline SVG trajectory and phase-portrait charts."
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

    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => output = Some(value.clone()),
            "--title" => options.title = value.clone(),
            "--start" => options.start = parse_f64(value)?,
            "--end" => options.end = parse_f64(value)?,
            "--step" => options.step = parse_f64(value)?,
            "--initial" => {
                let (id, number) =
                    parse_assignment_text(value).map_err(|error| error.to_string())?;
                initial_overrides.insert(id, number);
            }
            _ => return Err(help()),
        }
        index += 2;
    }
    options.initial_overrides = initial_overrides;

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let html = render_report(&world, &options).map_err(|error| error.to_string())?;

    let output_path = output.unwrap_or_else(|| default_output_path(bundle));
    fs::write(&output_path, &html)
        .map_err(|error| format!("failed to write {output_path}: {error}"))?;

    Ok(format!(
        "wrote report: {output_path} ({} bytes, {} state variable(s))\n",
        html.len(),
        world.state_ids().count()
    ))
}

fn default_output_path(bundle: &str) -> String {
    let stem = Path::new(bundle).file_stem().and_then(|stem| stem.to_str()).unwrap_or("report");
    format!("{stem}.report.html")
}

fn parse_f64(value: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}
