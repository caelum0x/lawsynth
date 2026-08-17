//! `lawsynth prep` — clean a dataset before discovery using the real
//! `lawsynth-preprocess` transforms.
//!
//! Each operation is honest: it only exposes a transform the preprocess crate
//! (or the validated `Dataset` boundary) actually implements. Operations are
//! applied in the order given on the command line, each producing a new
//! immutable `Dataset` plus a one-line provenance summary.

use std::fmt::Write as _;
use std::fs;

use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::{detrend_linear, moving_average, resample_linear_with_report};
use lawsynth_report::format_number;

use crate::read_numeric_dataset;

/// A single cleaning operation, in the order the user requested it.
enum Op {
    /// Centered moving average with the given half-window radius.
    Smooth(usize),
    /// Uniform re-gridding onto a constant timestep.
    Resample(f64),
    /// Remove a least-squares linear trend from every column.
    Detrend,
    /// Keep only rows whose timestamp falls in `[start, end]`.
    Trim(f64, f64),
    /// Drop columns whose values never change (zero range).
    DropConstant,
}

struct PrepArgs {
    input: String,
    time_column: String,
    output: String,
    ops: Vec<Op>,
}

/// Help text for `lawsynth prep`.
pub fn help() -> String {
    "lawsynth prep OBS.{csv,tsv,parquet} [--time COLUMN] --output CLEAN.csv \
[--trim START:END] [--drop-constant] [--detrend] [--smooth-window N] [--resample DT]\n\n\
Cleans a dataset before discovery using the real lawsynth-preprocess transforms. \
Operations apply in the order given, each on the previous result:\n\
  --trim START:END   keep only the usable time window [START, END]\n\
  --drop-constant    remove columns that never change (zero range)\n\
  --detrend          subtract a least-squares linear trend per column\n\
  --smooth-window N   centered moving average, N = half-window radius (N>=1)\n\
  --resample DT      linearly re-grid every column onto a uniform DT step\n\n\
Writes the cleaned CSV to --output and prints a summary of what changed. \
Defaults: --time time."
        .to_owned()
}

/// Runs the `prep` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let original = read_numeric_dataset(&args.input, &args.time_column)?;

    let rows_in = original.time().len();
    let cols_in = original.columns().len();
    let fingerprint_in = original.fingerprint();

    let mut summary = String::new();
    let _ = writeln!(summary, "prep {} -> {}", args.input, args.output);
    let _ = writeln!(
        summary,
        "  input : {rows_in} rows, {cols_in} column(s) [{}]",
        column_names(&original)
    );

    let mut dataset = original;
    if args.ops.is_empty() {
        summary.push_str("  (no operations requested; writing a validated copy)\n");
    }
    for op in &args.ops {
        let (next, line) = apply(&dataset, op)?;
        dataset = next;
        let _ = writeln!(summary, "  op    : {line}");
    }

    let rows_out = dataset.time().len();
    let cols_out = dataset.columns().len();
    let csv = dataset_to_csv(&dataset, &args.time_column);
    fs::write(&args.output, &csv)
        .map_err(|error| format!("failed to write {}: {error}", args.output))?;

    let _ = writeln!(
        summary,
        "  output: {rows_out} rows, {cols_out} column(s) [{}]",
        column_names(&dataset)
    );
    let _ = writeln!(
        summary,
        "  change: rows {rows_in} -> {rows_out}, columns {cols_in} -> {cols_out}, \
content fingerprint {fingerprint_in:016x} -> {:016x}",
        dataset.fingerprint()
    );
    Ok(summary)
}

fn apply(dataset: &Dataset, op: &Op) -> Result<(Dataset, String), String> {
    match op {
        Op::Smooth(radius) => {
            let (out, report) =
                moving_average(dataset, *radius).map_err(|error| error.to_string())?;
            Ok((
                out,
                format!(
                    "smooth-window radius={radius} (moving average), fingerprint {:016x} -> {:016x}",
                    report.input_fingerprint, report.output_fingerprint
                ),
            ))
        }
        Op::Resample(dt) => {
            let target = uniform_axis(dataset.time().values(), *dt)?;
            let points = target.len();
            let (out, report) =
                resample_linear_with_report(dataset, target).map_err(|error| error.to_string())?;
            Ok((
                out,
                format!(
                    "resample dt={} -> {points} uniform samples, fingerprint {:016x} -> {:016x}",
                    format_number(*dt),
                    report.input_fingerprint,
                    report.output_fingerprint
                ),
            ))
        }
        Op::Detrend => {
            let (out, report) = detrend_linear(dataset).map_err(|error| error.to_string())?;
            let mut slopes = report.slope.iter().collect::<Vec<_>>();
            slopes.sort_by(|a, b| a.0.cmp(b.0));
            let described = slopes
                .iter()
                .map(|(name, slope)| format!("{name}:{}", format_number(**slope)))
                .collect::<Vec<_>>()
                .join(", ");
            Ok((out, format!("detrend (removed linear slope per column: {described})")))
        }
        Op::Trim(start, end) => {
            let out = trim(dataset, *start, *end)?;
            Ok((
                out.0,
                format!(
                    "trim [{}, {}] kept {} of {} rows",
                    format_number(*start),
                    format_number(*end),
                    out.1,
                    dataset.time().len()
                ),
            ))
        }
        Op::DropConstant => {
            let (out, dropped) = drop_constant(dataset)?;
            let described = if dropped.is_empty() { "none".to_owned() } else { dropped.join(", ") };
            Ok((out, format!("drop-constant removed {} column(s): {described}", dropped.len())))
        }
    }
}

/// Builds a uniform in-range time axis stepping by `dt` from the first sample.
fn uniform_axis(source: &[f64], dt: f64) -> Result<TimeAxis, String> {
    if !(dt.is_finite() && dt > 0.0) {
        return Err(format!("resample step must be a positive number (got {dt})"));
    }
    let start = source[0];
    let last = source[source.len() - 1];
    // A tolerance keeps the final grid point in-range against float drift so the
    // preprocess resampler does not reject it as out-of-bounds.
    let tolerance = dt * 1e-9;
    let mut values = Vec::new();
    let mut index = 0usize;
    loop {
        let time = start + dt * index as f64;
        if time > last + tolerance {
            break;
        }
        values.push(time.min(last));
        index += 1;
    }
    if values.len() < 2 {
        return Err("resample step is too large to produce at least two samples".to_owned());
    }
    TimeAxis::new(values).map_err(|error| error.to_string())
}

/// Keeps only the rows whose timestamp lies within `[start, end]`.
fn trim(dataset: &Dataset, start: f64, end: f64) -> Result<(Dataset, usize), String> {
    if !(start.is_finite() && end.is_finite()) || start >= end {
        return Err("trim window must be START:END with START < END and finite bounds".to_owned());
    }
    let times = dataset.time().values();
    let kept: Vec<usize> =
        (0..times.len()).filter(|&i| times[i] >= start && times[i] <= end).collect();
    if kept.len() < 2 {
        return Err(format!(
            "trim [{}, {}] keeps fewer than two rows",
            format_number(start),
            format_number(end)
        ));
    }
    let axis = TimeAxis::new(kept.iter().map(|&i| times[i]).collect())
        .map_err(|error| error.to_string())?;
    let columns: Vec<NumericColumn> = dataset
        .columns()
        .values()
        .map(|column| NumericColumn {
            id: column.id.clone(),
            values: kept.iter().map(|&i| column.values[i]).collect(),
            unit: column.unit.clone(),
        })
        .collect();
    let count = kept.len();
    Dataset::new(axis, columns).map(|dataset| (dataset, count)).map_err(|error| error.to_string())
}

/// Removes every column whose values never change (zero numeric range).
fn drop_constant(dataset: &Dataset) -> Result<(Dataset, Vec<String>), String> {
    let mut kept = Vec::new();
    let mut dropped = Vec::new();
    for column in dataset.columns().values() {
        let min = column.values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = column.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        // Scale the tolerance so genuinely varying-but-tiny columns survive.
        let tolerance = max.abs().max(min.abs()).max(1.0) * 1e-12;
        if (max - min).abs() <= tolerance {
            dropped.push(column.id.to_string());
        } else {
            kept.push(column.clone());
        }
    }
    if kept.is_empty() {
        return Err("drop-constant would remove every column; nothing left to discover".to_owned());
    }
    let axis =
        TimeAxis::new(dataset.time().values().to_vec()).map_err(|error| error.to_string())?;
    Dataset::new(axis, kept).map(|dataset| (dataset, dropped)).map_err(|error| error.to_string())
}

/// Serializes a dataset as stable numeric CSV with the given time header.
fn dataset_to_csv(dataset: &Dataset, time_column: &str) -> String {
    let mut csv = String::from(time_column);
    for id in dataset.columns().keys() {
        let _ = write!(csv, ",{}", id.as_str());
    }
    csv.push('\n');
    let times = dataset.time().values();
    for (row, time) in times.iter().enumerate() {
        let _ = write!(csv, "{time:.12e}");
        for column in dataset.columns().values() {
            let _ = write!(csv, ",{:.12e}", column.values[row]);
        }
        csv.push('\n');
    }
    csv
}

fn column_names(dataset: &Dataset) -> String {
    dataset.columns().keys().map(Identifier::as_str).collect::<Vec<_>>().join(",")
}

fn parse(arguments: &[String]) -> Result<PrepArgs, String> {
    let Some(input) = arguments.first() else {
        return Err(help());
    };
    if input.starts_with('-') {
        return Err(help());
    }
    let mut time_column = "time".to_owned();
    let mut output = None;
    let mut ops = Vec::new();
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        match option {
            "--detrend" => {
                ops.push(Op::Detrend);
                index += 1;
                continue;
            }
            "--drop-constant" => {
                ops.push(Op::DropConstant);
                index += 1;
                continue;
            }
            _ => {}
        }
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--time" => time_column = value.clone(),
            "--output" => output = Some(value.clone()),
            "--smooth-window" => ops.push(Op::Smooth(parse_radius(value)?)),
            "--resample" => ops.push(Op::Resample(parse_f64(value)?)),
            "--trim" => {
                let (start, end) = parse_trim(value)?;
                ops.push(Op::Trim(start, end));
            }
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(PrepArgs {
        input: input.clone(),
        time_column,
        output: output.ok_or("missing required --output CLEAN.csv")?,
        ops,
    })
}

fn parse_trim(value: &str) -> Result<(f64, f64), String> {
    let (start, end) =
        value.split_once(':').ok_or_else(|| "expected --trim START:END".to_owned())?;
    Ok((parse_f64(start)?, parse_f64(end)?))
}

fn parse_radius(value: &str) -> Result<usize, String> {
    let radius: usize = value.parse().map_err(|_| format!("invalid window radius '{value}'"))?;
    if radius == 0 {
        return Err("--smooth-window radius must be at least 1".to_owned());
    }
    Ok(radius)
}

fn parse_f64(value: &str) -> Result<f64, String> {
    let number: f64 = value.parse().map_err(|_| format!("invalid number '{value}'"))?;
    if number.is_finite() { Ok(number) } else { Err(format!("number '{value}' must be finite")) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn sample() -> Dataset {
        Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0, 4.0]).unwrap(),
            [
                NumericColumn::new(id("x"), vec![0.0, 3.0, 0.0, 3.0, 0.0]),
                NumericColumn::new(id("c"), vec![7.0, 7.0, 7.0, 7.0, 7.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn trim_keeps_only_the_requested_window() {
        let (trimmed, kept) = trim(&sample(), 1.0, 3.0).unwrap();
        assert_eq!(kept, 3);
        assert_eq!(trimmed.time().values(), &[1.0, 2.0, 3.0]);
    }

    #[test]
    fn drop_constant_removes_flat_columns() {
        let (out, dropped) = drop_constant(&sample()).unwrap();
        assert_eq!(dropped, vec!["c".to_owned()]);
        assert!(out.columns().contains_key(&id("x")));
        assert!(!out.columns().contains_key(&id("c")));
    }

    #[test]
    fn uniform_axis_stays_in_range() {
        let axis = uniform_axis(&[0.0, 1.0, 2.0, 3.3], 1.0).unwrap();
        assert_eq!(axis.values()[0], 0.0);
        assert!(*axis.values().last().unwrap() <= 3.3);
    }

    #[test]
    fn csv_round_trips_the_time_header() {
        let csv = dataset_to_csv(&sample(), "t");
        assert!(csv.starts_with("t,c,x\n"));
        assert_eq!(csv.lines().count(), 6);
    }
}
