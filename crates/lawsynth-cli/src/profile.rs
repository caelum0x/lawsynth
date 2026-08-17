//! `lawsynth profile` — a "know your data" command that inspects a dataset
//! *before* discovery. It reports per-column statistics (type, min/max/mean/std,
//! count, missing) and dataset-level structure (row count, time monotonicity and
//! sampling regularity, constant/degenerate columns) plus quality warnings, in
//! clear text or `--json`. It pairs with `validate` and `discover` in the core
//! loop: understand the observations, then model them.
//!
//! All numbers come from the real `lawsynth-profile` crate over the same
//! `lawsynth-data` boundary that `discover` uses, so a profile is a faithful
//! preview of what discovery will actually see. The loader rejects non-finite
//! values at ingestion, so a fully missing/NaN column surfaces as a load error
//! rather than a silent partial dataset; within a loaded dataset `missing` is 0.

use std::fmt::Write as _;

use lawsynth_core::Identifier;
use lawsynth_profile::{ColumnProfile, DatasetProfile, profile};

use crate::read_numeric_dataset;

/// Emits a concise stable profile summary for command-line callers.
pub fn profile_summary(profile: &DatasetProfile) -> String {
    format!("samples={}, columns={}\n", profile.samples, profile.columns.len())
}

/// Minimum sample count below which discovery is unlikely to be well-posed.
const MIN_RECOMMENDED_SAMPLES: usize = 10;

/// Help text for `lawsynth profile`.
pub fn help() -> String {
    "lawsynth profile OBSERVATIONS.{csv,tsv,parquet} [--time COLUMN] [--json]\n\n\
Inspects a dataset before discovery: per-column type, min/max/mean/std, count, \
and missing values, plus dataset-level row count, time monotonicity and sampling \
regularity, constant/degenerate columns, and quality warnings. Defaults to the \
'time' column; override with --time. Use --json for machine-readable output."
        .to_owned()
}

/// Runs the `profile` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(input) = arguments.first() else {
        return Err(help());
    };
    let mut time_column = "time".to_owned();
    let mut as_json = false;
    let mut index = 1;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--json" => {
                as_json = true;
                index += 1;
            }
            "--time" => {
                time_column = arguments
                    .get(index + 1)
                    .ok_or_else(|| "missing value for --time".to_owned())?
                    .clone();
                index += 2;
            }
            other => return Err(format!("unexpected argument '{other}'\n\n{}", help())),
        }
    }

    let dataset = read_numeric_dataset(input, &time_column)?;
    let report = profile(&dataset).map_err(|error| error.to_string())?;
    let warnings = warnings(&report);
    if as_json {
        Ok(render_json(input, &time_column, &report, &warnings))
    } else {
        Ok(render_text(input, &time_column, &report, &warnings))
    }
}

/// Classifies a column for the human-facing `type` field.
fn column_type(is_constant: bool) -> &'static str {
    if is_constant { "constant" } else { "numeric" }
}

/// Collects dataset-level and per-column quality warnings, deterministically.
fn warnings(report: &DatasetProfile) -> Vec<String> {
    let mut warnings = Vec::new();
    if report.samples < MIN_RECOMMENDED_SAMPLES {
        warnings.push(format!(
            "only {} sample(s); discovery typically needs >= {MIN_RECOMMENDED_SAMPLES}",
            report.samples
        ));
    }
    if !report.time.is_regular {
        warnings.push("time sampling is irregular (non-uniform step)".to_owned());
    }
    // Columns are iterated in the profile's stable, lexicographic id order.
    for (id, quality) in &report.quality {
        if quality.is_constant {
            warnings.push(format!("column '{id}' is constant (degenerate)"));
        }
        if !quality.outlier_indices.is_empty() {
            warnings.push(format!(
                "column '{id}' has {} Tukey-IQR outlier(s)",
                quality.outlier_indices.len()
            ));
        }
    }
    for (id, missingness) in &report.missingness {
        if missingness.missing > 0 {
            warnings.push(format!("column '{id}' has {} missing value(s)", missingness.missing));
        }
    }
    warnings
}

fn std_dev(column: &ColumnProfile) -> f64 {
    column.variance.max(0.0).sqrt()
}

fn render_text(
    source: &str,
    time_column: &str,
    report: &DatasetProfile,
    warnings: &[String],
) -> String {
    let mut out = String::new();
    let names: Vec<&str> = report.columns.keys().map(Identifier::as_str).collect();
    let _ = writeln!(out, "dataset profile: {source}");
    let _ = writeln!(out, "  rows:        {}", report.samples);
    let _ = writeln!(out, "  columns:     {}  ({})", report.columns.len(), names.join(", "));
    let _ = writeln!(out, "  fingerprint: {:#018x}", report.fingerprint);

    let _ = writeln!(out, "\ntime column '{time_column}':");
    let _ = writeln!(out, "  range:       {:.6e} .. {:.6e}", report.time.start, report.time.end);
    let _ =
        writeln!(out, "  step:        {:.6e} ({})", report.time.nominal_step, uniformity(report));
    // The data boundary guarantees strictly increasing timestamps.
    let _ = writeln!(out, "  ordering:    strictly increasing (monotonic)");
    let _ = writeln!(out, "  regular:     {}", yes_no(report.time.is_regular));

    let _ = writeln!(out, "\ncolumns:");
    let _ = writeln!(
        out,
        "  {:<12}  {:<9}  {:>6}  {:>7}  {:>13}  {:>13}  {:>13}  {:>13}",
        "name", "type", "count", "missing", "min", "max", "mean", "std"
    );
    for (id, column) in &report.columns {
        let is_constant = report.quality.get(id).map(|q| q.is_constant).unwrap_or(false);
        let missing = report.missingness.get(id).map(|m| m.missing).unwrap_or(0);
        let _ = writeln!(
            out,
            "  {:<12}  {:<9}  {:>6}  {:>7}  {:>13.6e}  {:>13.6e}  {:>13.6e}  {:>13.6e}",
            id.as_str(),
            column_type(is_constant),
            report.samples,
            missing,
            column.minimum,
            column.maximum,
            column.mean,
            std_dev(column),
        );
    }

    let _ = writeln!(out, "\nwarnings:");
    if warnings.is_empty() {
        let _ = writeln!(out, "  none");
    } else {
        for warning in warnings {
            let _ = writeln!(out, "  - {warning}");
        }
    }
    out
}

fn render_json(
    source: &str,
    time_column: &str,
    report: &DatasetProfile,
    warnings: &[String],
) -> String {
    let mut out = String::from("{\n");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let _ = writeln!(out, "  \"rows\": {},", report.samples);
    let _ = writeln!(
        out,
        "  \"fingerprint\": {},",
        json_string(&format!("{:#018x}", report.fingerprint))
    );
    let _ = writeln!(out, "  \"time\": {{");
    let _ = writeln!(out, "    \"column\": {},", json_string(time_column));
    let _ = writeln!(out, "    \"start\": {},", json_number(report.time.start));
    let _ = writeln!(out, "    \"end\": {},", json_number(report.time.end));
    let _ = writeln!(out, "    \"step\": {},", json_number(report.time.nominal_step));
    let _ = writeln!(out, "    \"regular\": {},", report.time.is_regular);
    let _ = writeln!(out, "    \"monotonic\": \"strictly_increasing\"");
    let _ = writeln!(out, "  }},");
    let _ = writeln!(out, "  \"columns\": [");
    let mut first = true;
    for (id, column) in &report.columns {
        if !first {
            out.push_str(",\n");
        }
        first = false;
        let is_constant = report.quality.get(id).map(|q| q.is_constant).unwrap_or(false);
        let missing = report.missingness.get(id).map(|m| m.missing).unwrap_or(0);
        let _ = write!(
            out,
            "    {{ \"name\": {}, \"type\": {}, \"count\": {}, \"missing\": {}, \
\"min\": {}, \"max\": {}, \"mean\": {}, \"std\": {} }}",
            json_string(id.as_str()),
            json_string(column_type(is_constant)),
            report.samples,
            missing,
            json_number(column.minimum),
            json_number(column.maximum),
            json_number(column.mean),
            json_number(std_dev(column)),
        );
    }
    if !first {
        out.push('\n');
    }
    let _ = writeln!(out, "  ],");
    let _ = write!(out, "  \"warnings\": [");
    for (index, warning) in warnings.iter().enumerate() {
        if index == 0 {
            out.push('\n');
        }
        let _ = write!(out, "    {}", json_string(warning));
        if index + 1 < warnings.len() {
            out.push(',');
        }
        out.push('\n');
    }
    if warnings.is_empty() {
        out.push_str("]\n");
    } else {
        out.push_str("  ]\n");
    }
    out.push_str("}\n");
    out
}

fn uniformity(report: &DatasetProfile) -> &'static str {
    if report.time.is_regular { "uniform" } else { "non-uniform" }
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

/// Emits a finite float as a JSON number using a stable scientific format.
fn json_number(value: f64) -> String {
    format!("{value:.6e}")
}

/// Escapes a string for JSON output (std-only, no external serializer).
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    /// A small dataset with an irregular time axis and one constant column,
    /// exercising both the non-uniform-sampling and degenerate-column paths.
    fn irregular_dataset() -> Dataset {
        Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 3.0, 6.0]).unwrap(),
            [
                NumericColumn::new(id("x"), vec![1.0, 2.0, 3.0, 4.0]),
                NumericColumn::new(id("k"), vec![5.0, 5.0, 5.0, 5.0]),
            ],
        )
        .unwrap()
    }

    #[test]
    fn text_report_lists_columns_and_flags_constant() {
        let report = profile(&irregular_dataset()).unwrap();
        let warnings = warnings(&report);
        let text = render_text("obs.csv", "time", &report, &warnings);
        assert!(text.contains("rows:        4"));
        assert!(text.contains("strictly increasing"));
        assert!(text.contains("column 'k' is constant"));
        // Only 4 samples => too-few-samples warning as well.
        assert!(text.contains("only 4 sample(s)"));
    }

    #[test]
    fn json_report_is_structurally_stable() {
        let report = profile(&irregular_dataset()).unwrap();
        let warnings = warnings(&report);
        let json = render_json("obs.csv", "time", &report, &warnings);
        assert!(json.starts_with("{\n"));
        assert!(json.trim_end().ends_with('}'));
        assert!(json.contains("\"regular\": false"));
        assert!(json.contains("\"name\": \"k\""));
        assert!(json.contains("\"monotonic\": \"strictly_increasing\""));
    }

    #[test]
    fn json_string_escapes_control_characters() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("a\tb"), "\"a\\tb\"");
    }
}
