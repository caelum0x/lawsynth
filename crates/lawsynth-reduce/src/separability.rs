//! Additive / multiplicative separability detection.
//!
//! For a bipartition `(A, B)` of the input variables, `f = g(A) + h(B)` holds iff
//! every cross mixed partial `∂²f/∂x_i∂x_j` (`i∈A, j∈B`) vanishes; multiplicative
//! separability is the same statement on `log|f|`. Both are *screened* with the
//! numerical mixed partial (the AI-Feynman probe, but deterministic — the field's
//! own finite differences replace a trained network) and then *verified* by
//! reconstructing the data from the reduced form and measuring the residual.

use std::collections::BTreeMap;

use crate::ReduceError;
use crate::config::ReduceConfig;
use crate::grid::{GridField, mean, range, rms};
use crate::report::{Separability, SeparabilityKind, confidence_from_residual};

/// Detects every additive/multiplicative separability that passes both the
/// screen and the reconstruction verification, best (highest confidence) first.
pub(crate) fn detect(
    field: &GridField,
    config: &ReduceConfig,
) -> Result<Vec<Separability>, ReduceError> {
    let mut found = Vec::new();
    let n = field.ndim();
    if n < 2 {
        return Ok(found);
    }

    // Precompute per-axis first partials once; mixed partials reuse them.
    let partials: Vec<GridField> =
        (0..n).map(|axis| field.partial(axis)).collect::<Result<_, _>>()?;

    // Optional log-domain field for the multiplicative test.
    let log_field = build_log_field(field, config);
    let log_partials = match &log_field {
        Some(logf) => Some((0..n).map(|axis| logf.partial(axis)).collect::<Result<Vec<_>, _>>()?),
        None => None,
    };

    for (group_a, group_b) in bipartitions(n) {
        // Additive.
        if let Some(sep) =
            evaluate(field, &partials, &group_a, &group_b, SeparabilityKind::Additive, config)?
        {
            found.push(sep);
        }
        // Multiplicative (only when a valid log field exists).
        if let (Some(logf), Some(logp)) = (&log_field, &log_partials) {
            if let Some(sep) =
                evaluate_multiplicative(field, logf, logp, &group_a, &group_b, config)?
            {
                found.push(sep);
            }
        }
    }

    // Deterministic ordering: highest confidence first, then by partition.
    found.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.group_a.cmp(&b.group_a))
            .then_with(|| a.group_b.cmp(&b.group_b))
            .then_with(|| (a.kind as u8).cmp(&(b.kind as u8)))
    });
    Ok(found)
}

/// Enumerates each unordered bipartition of `n` variables into two non-empty
/// groups exactly once (variable 0 is always placed in group A).
fn bipartitions(n: usize) -> Vec<(Vec<usize>, Vec<usize>)> {
    let mut out = Vec::new();
    let full = 1usize << n;
    for mask in 1..full {
        if mask & 1 == 0 || mask == full - 1 {
            continue; // variable 0 must be in A, and B must be non-empty
        }
        let group_a: Vec<usize> = (0..n).filter(|&i| mask & (1 << i) != 0).collect();
        let group_b: Vec<usize> = (0..n).filter(|&i| mask & (1 << i) == 0).collect();
        out.push((group_a, group_b));
    }
    out
}

/// Screens + verifies additive separability across `(group_a, group_b)`.
fn evaluate(
    field: &GridField,
    partials: &[GridField],
    group_a: &[usize],
    group_b: &[usize],
    kind: SeparabilityKind,
    config: &ReduceConfig,
) -> Result<Option<Separability>, ReduceError> {
    let field_range = range(&field.values);
    if field_range <= config.constant_field_tol * mean(&field.values).abs().max(1.0) {
        return Ok(None); // constant field: nothing to separate
    }

    let screen = cross_mixed_partial_residual(field, partials, group_a, group_b, field_range)?;
    if screen > config.separability_screen_tol {
        return Ok(None);
    }

    let reconstruction = additive_reconstruction_residual(field, group_a, group_b);
    if reconstruction > config.additive_tol {
        return Ok(None);
    }

    Ok(Some(Separability {
        kind,
        group_a: names(field, group_a),
        group_b: names(field, group_b),
        screening_residual: screen,
        reconstruction_residual: reconstruction,
        confidence: confidence_from_residual(reconstruction),
    }))
}

/// Screens + verifies multiplicative separability using the log-domain field.
fn evaluate_multiplicative(
    field: &GridField,
    log_field: &GridField,
    log_partials: &[GridField],
    group_a: &[usize],
    group_b: &[usize],
    config: &ReduceConfig,
) -> Result<Option<Separability>, ReduceError> {
    let log_range = range(&log_field.values);
    if log_range <= config.constant_field_tol {
        return Ok(None);
    }

    let screen =
        cross_mixed_partial_residual(log_field, log_partials, group_a, group_b, log_range)?;
    if screen > config.separability_screen_tol {
        return Ok(None);
    }

    // Reconstruct in log space, exponentiate, and score against the original f.
    let sign = field.values[0].signum();
    let log_hat = additive_reconstruction(log_field, group_a, group_b);
    let field_mean = mean(&field.values);
    let mut num = 0.0;
    let mut den = 0.0;
    for (cell, &f) in field.values.iter().enumerate() {
        let f_hat = sign * log_hat[cell].exp();
        num += (f - f_hat) * (f - f_hat);
        den += (f - field_mean) * (f - field_mean);
    }
    let reconstruction = if den > 0.0 { (num / den).sqrt() } else { 0.0 };
    if reconstruction > config.multiplicative_tol {
        return Ok(None);
    }

    Ok(Some(Separability {
        kind: SeparabilityKind::Multiplicative,
        group_a: names(field, group_a),
        group_b: names(field, group_b),
        screening_residual: screen,
        reconstruction_residual: reconstruction,
        confidence: confidence_from_residual(reconstruction),
    }))
}

/// Largest normalized cross mixed-partial magnitude over all `(i∈A, j∈B)` pairs.
fn cross_mixed_partial_residual(
    field: &GridField,
    partials: &[GridField],
    group_a: &[usize],
    group_b: &[usize],
    field_range: f64,
) -> Result<f64, ReduceError> {
    let mut worst = 0.0f64;
    for &i in group_a {
        for &j in group_b {
            let mixed = partials[i].partial(j)?;
            let interior = field.interior_cells(&[i, j]);
            let sampled: Vec<f64> = interior.iter().map(|&c| mixed.values[c]).collect();
            let axis_scale =
                (range(&field.axes[i].coords) * range(&field.axes[j].coords)).max(1e-30);
            let scale = (field_range / axis_scale).max(1e-30);
            let residual = rms(&sampled) / scale;
            worst = worst.max(residual);
        }
    }
    Ok(worst)
}

/// Relative RMS residual of the additive (two-way main-effects) reconstruction.
fn additive_reconstruction_residual(
    field: &GridField,
    group_a: &[usize],
    group_b: &[usize],
) -> f64 {
    let hat = additive_reconstruction(field, group_a, group_b);
    let field_mean = mean(&field.values);
    let mut num = 0.0;
    let mut den = 0.0;
    for (cell, &v) in field.values.iter().enumerate() {
        num += (v - hat[cell]) * (v - hat[cell]);
        den += (v - field_mean) * (v - field_mean);
    }
    if den > 0.0 { (num / den).sqrt() } else { 0.0 }
}

/// `f̂ = mean_B f + mean_A f − mean f`, the additive main-effects decomposition.
fn additive_reconstruction(field: &GridField, group_a: &[usize], group_b: &[usize]) -> Vec<f64> {
    let grand = mean(&field.values);
    // mean over B (grouped by the A-index key) and mean over A (grouped by B).
    let mean_over_b = subset_means(field, group_a); // varies with A indices
    let mean_over_a = subset_means(field, group_b); // varies with B indices
    (0..field.len())
        .map(|cell| {
            let a_key = subset_key(field, cell, group_a);
            let b_key = subset_key(field, cell, group_b);
            mean_over_b[&a_key] + mean_over_a[&b_key] - grand
        })
        .collect()
}

/// Mean of the field grouped by the multi-index over `axes` (the other axes are
/// averaged out). Keys are deterministic mixed-radix encodings.
fn subset_means(field: &GridField, axes: &[usize]) -> BTreeMap<u64, f64> {
    let mut sums: BTreeMap<u64, (f64, usize)> = BTreeMap::new();
    for (cell, &v) in field.values.iter().enumerate() {
        let key = subset_key(field, cell, axes);
        let entry = sums.entry(key).or_insert((0.0, 0));
        entry.0 += v;
        entry.1 += 1;
    }
    sums.into_iter().map(|(k, (sum, count))| (k, sum / count as f64)).collect()
}

/// Mixed-radix key of `cell`'s indices restricted to `axes`.
fn subset_key(field: &GridField, cell: usize, axes: &[usize]) -> u64 {
    let mut key = 0u64;
    for &axis in axes {
        let len = field.axes[axis].coords.len() as u64;
        key = key * len + field.axis_index(cell, axis) as u64;
    }
    key
}

/// Builds `ln|f|` when `f` is sign-consistent and bounded away from zero.
fn build_log_field(field: &GridField, config: &ReduceConfig) -> Option<GridField> {
    let first_sign = field.values[0].signum();
    for &v in &field.values {
        if v.abs() < config.multiplicative_floor || v.signum() != first_sign {
            return None;
        }
    }
    Some(GridField {
        axes: field.axes.clone(),
        values: field.values.iter().map(|v| v.abs().ln()).collect(),
    })
}

fn names(field: &GridField, group: &[usize]) -> Vec<String> {
    group.iter().map(|&i| field.axes[i].name.clone()).collect()
}
