//! `lawsynth pde` — evolution-PDE discovery (PDE-FIND) from a 1-D field.
//!
//! Given snapshots of a 1-D field `u(x, t)` on a regular space–time grid, this
//! command discovers an evolution law `u_t = F(u, u_x, u_xx, …)` by central
//! finite differences plus sparse regression. See [`lawsynth_pde::discover_pde`].
//!
//! # Input format
//!
//! The field CSV is a **plain rectangular numeric grid with no header**: each row
//! is a time snapshot, each comma-separated column is a spatial point. Every row
//! must have the same number of columns. `--dx` and `--dt` give the (uniform)
//! spatial and temporal grid steps. This is deliberately distinct from the
//! trajectory-dataset CSV the other commands read (which carries a named `time`
//! column and one column per state), because a PDE field is a 2-D grid, not a
//! table of named state series.

use std::fmt::Write as _;

use lawsynth_pde::{PdeConfig, PdeModel, discover_pde};
use lawsynth_report::format_number;

use crate::analysis::{json_string, parse_positive, parse_usize};

/// Help text for `lawsynth pde`.
pub fn help() -> String {
    "lawsynth pde FIELD.csv --dx DX --dt DT [--degree D] [--order M] [--threshold T] [--json]\n\n\
Discovers a 1-D evolution PDE u_t = F(u, u_x, u_xx, …) from a field grid \
(PDE-FIND): spatial and temporal derivatives are estimated with central finite \
differences and the flattened u_t is sparse-regressed onto a differential-term \
library. FIELD.csv is a plain rectangular numeric grid with NO header — rows are \
time snapshots, columns are spatial points. --dx and --dt are the uniform grid \
steps; --degree is the maximum field power (u, u², …), --order the maximum \
spatial-derivative order (u_x, u_xx, u_xxx), --threshold the relative sparsity \
cutoff. --json emits the term coefficients. Finite differencing amplifies noise, \
so recovery is noise-sensitive and needs a resolved grid; finer grids tighten \
the coefficients."
        .to_owned()
}

/// Runs the `pde` command.
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

    let mut dx = None;
    let mut dt = None;
    let mut degree = None;
    let mut order = None;
    let mut threshold = None;
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
            "--dx" => dx = Some(parse_positive(value, "--dx")?),
            "--dt" => dt = Some(parse_positive(value, "--dt")?),
            "--degree" => degree = Some(parse_usize(value, "--degree")?),
            "--order" => order = Some(parse_usize(value, "--order")?),
            "--threshold" => threshold = Some(parse_positive(value, "--threshold")?),
            _ => return Err(help()),
        }
        index += 2;
    }

    let dx = dx.ok_or_else(|| "--dx DX is required".to_owned())?;
    let dt = dt.ok_or_else(|| "--dt DT is required".to_owned())?;

    let field = read_field(input)?;

    let mut config = PdeConfig::default();
    if let Some(degree) = degree {
        config = config.with_u_degree(degree);
    }
    if let Some(order) = order {
        config = config.with_derivative_order(order);
    }
    if let Some(threshold) = threshold {
        config.sparse.threshold = threshold;
    }

    let model = discover_pde(&field, dx, dt, &config).map_err(|error| error.to_string())?;

    let (nt, nx) = (field.len(), field.first().map(Vec::len).unwrap_or(0));
    if as_json {
        Ok(render_json(input, &model, nt, nx))
    } else {
        Ok(render_text(input, &model, nt, nx))
    }
}

/// Reads a headerless rectangular numeric CSV grid into `field[t][x]`.
///
/// Blank lines are skipped; every non-blank row must parse to a finite float and
/// all rows must share the same column count.
fn read_field(path: &str) -> Result<Vec<Vec<f64>>, String> {
    let text = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    let mut field: Vec<Vec<f64>> = Vec::new();
    for (line_number, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut row = Vec::new();
        for cell in trimmed.split(',') {
            let value: f64 = cell.trim().parse().map_err(|_| {
                format!("field cell '{}' on line {} is not a number", cell.trim(), line_number + 1)
            })?;
            if !value.is_finite() {
                return Err(format!("field value on line {} is not finite", line_number + 1));
            }
            row.push(value);
        }
        if let Some(first) = field.first()
            && row.len() != first.len()
        {
            return Err(format!(
                "field is not rectangular: line {} has {} column(s), expected {}",
                line_number + 1,
                row.len(),
                first.len()
            ));
        }
        field.push(row);
    }
    if field.is_empty() {
        return Err("field CSV has no numeric rows".to_owned());
    }
    Ok(field)
}

/// Human-facing report.
fn render_text(source: &str, model: &PdeModel, nt: usize, nx: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "PDE discovery from {source}");
    let _ = writeln!(
        out,
        "  grid:     {nt} x {nx} (dx={}, dt={})",
        format_number(model.dx),
        format_number(model.dt)
    );
    let _ = writeln!(out, "  interior: {} point(s) fed the regression", model.interior_points);
    let _ = writeln!(
        out,
        "  library:  max field power {}, max derivative order {}",
        model.max_u_degree, model.max_derivative_order
    );
    out.push('\n');

    let _ = writeln!(out, "Discovered evolution law:");
    let _ = writeln!(out, "  {}", model.describe());
    out.push('\n');
    let _ = writeln!(out, "Active terms (coefficient · u^p · D_m):");
    let mut any = false;
    for term in model.active_terms() {
        any = true;
        let _ = writeln!(out, "  {:<8} {}", term.label, format_number(term.coefficient));
    }
    if !any {
        let _ = writeln!(out, "  (every candidate term was thresholded out)");
    }
    let _ = writeln!(out, "  residual SS: {}", format_number(model.residual_sum_squares));
    out.push('\n');
    let _ = writeln!(
        out,
        "note: finite-difference PDE-FIND — differentiating the field amplifies noise, \
so recovery is noise-sensitive and needs a resolved grid; finer grids tighten the \
coefficients. It writes no .lsworld bundle."
    );
    out
}

/// Stable, machine-readable report.
fn render_json(source: &str, model: &PdeModel, nt: usize, nx: usize) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{{");
    let _ = writeln!(out, "  \"method\": \"pde-find\",");
    let _ = writeln!(out, "  \"source\": {},", json_string(source));
    let _ = writeln!(out, "  \"variable\": {},", json_string(model.variable.as_str()));
    let _ = writeln!(out, "  \"time_snapshots\": {nt},");
    let _ = writeln!(out, "  \"spatial_points\": {nx},");
    let _ = writeln!(out, "  \"dx\": {:.17e},", model.dx);
    let _ = writeln!(out, "  \"dt\": {:.17e},", model.dt);
    let _ = writeln!(out, "  \"interior_points\": {},", model.interior_points);
    let _ = writeln!(out, "  \"residual_sum_squares\": {:.17e},", model.residual_sum_squares);
    let _ = writeln!(out, "  \"law\": {},", json_string(&model.describe()));
    let terms: Vec<String> = model
        .terms
        .iter()
        .map(|term| {
            format!(
                "{{\"label\": {}, \"u_power\": {}, \"derivative_order\": {}, \
\"coefficient\": {:.17e}}}",
                json_string(&term.label),
                term.u_power,
                term.derivative_order,
                term.coefficient
            )
        })
        .collect();
    let _ = writeln!(out, "  \"terms\": [{}]", terms.join(", "));
    let _ = writeln!(out, "}}");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_documents_the_grid_format() {
        let help = help();
        assert!(help.contains("--dx"));
        assert!(help.contains("NO header"));
        assert!(help.contains("u_t"));
    }

    #[test]
    fn requires_dx_and_dt() {
        let error = run(&["field.csv".to_owned()]).unwrap_err();
        assert!(error.contains("--dx") || error.contains("pde"), "error: {error}");
    }

    #[test]
    fn rejects_a_ragged_grid() {
        let directory =
            std::env::temp_dir().join(format!("lawsynth-pde-ragged-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("ragged.csv");
        std::fs::write(&path, "1,2,3\n4,5\n").unwrap();
        let error = read_field(path.to_str().unwrap()).unwrap_err();
        assert!(error.contains("not rectangular"), "error: {error}");
        std::fs::remove_dir_all(&directory).unwrap();
    }
}
