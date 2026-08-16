use crate::{CausalConfig, CausalError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrangerResult {
    pub lag: usize,
    pub restricted_sse: f64,
    pub unrestricted_sse: f64,
    pub f_statistic: f64,
    pub observations: usize,
}
pub fn granger_test(cause: &[f64], effect: &[f64], config: CausalConfig) -> Result<GrangerResult> {
    let config = config.validate()?;
    if cause.len() != effect.len() {
        return Err(CausalError::LengthMismatch {
            expected: effect.len(),
            actual: cause.len(),
        });
    }
    if cause.len() < config.min_samples {
        return Err(CausalError::InsufficientSamples {
            required: config.min_samples,
            actual: cause.len(),
        });
    }
    if cause.iter().chain(effect).any(|v| !v.is_finite()) {
        return Err(CausalError::InvalidParameter("series"));
    }
    let n = cause.len() - config.max_lag;
    let mut y = Vec::with_capacity(n);
    let mut restricted = Vec::with_capacity(n);
    let mut unrestricted = Vec::with_capacity(n);
    for t in config.max_lag..cause.len() {
        y.push(effect[t]);
        let mut r = vec![1.0];
        for j in 1..=config.max_lag {
            r.push(effect[t - j]);
        }
        let mut u = r.clone();
        for j in 1..=config.max_lag {
            u.push(cause[t - j]);
        }
        restricted.push(r);
        unrestricted.push(u);
    }
    let rsse = sse(&restricted, &y, config.singular_tolerance)?;
    let usse = sse(&unrestricted, &y, config.singular_tolerance)?;
    let q = config.max_lag as f64;
    let denominator_df = n as f64 - unrestricted[0].len() as f64;
    if denominator_df <= 0.0 {
        return Err(CausalError::InsufficientSamples {
            required: 2 * config.max_lag + 2,
            actual: n + config.max_lag,
        });
    }
    let f_statistic =
        ((rsse - usse).max(0.0) / q) / (usse.max(config.singular_tolerance) / denominator_df);
    Ok(GrangerResult {
        lag: config.max_lag,
        restricted_sse: rsse,
        unrestricted_sse: usse,
        f_statistic,
        observations: n,
    })
}
fn sse(x: &[Vec<f64>], y: &[f64], tol: f64) -> Result<f64> {
    let p = x[0].len();
    let mut a = vec![vec![0.0; p + 1]; p];
    for i in 0..p {
        for j in 0..p {
            a[i][j] = x.iter().map(|row| row[i] * row[j]).sum();
        }
        a[i][p] = x.iter().zip(y).map(|(row, v)| row[i] * v).sum();
    }
    for col in 0..p {
        let pivot = (col..p)
            .max_by(|&i, &j| a[i][col].abs().partial_cmp(&a[j][col].abs()).unwrap())
            .unwrap();
        if a[pivot][col].abs() <= tol {
            return Err(CausalError::SingularDesign);
        }
        a.swap(col, pivot);
        let d = a[col][col];
        for v in &mut a[col][col..] {
            *v /= d;
        }
        for row in 0..p {
            if row != col {
                let factor = a[row][col];
                let pivot_tail = a[col][col..=p].to_vec();
                for (target, pivot) in a[row][col..=p].iter_mut().zip(pivot_tail) {
                    *target -= factor * pivot;
                }
            }
        }
    }
    let beta: Vec<f64> = a.iter().map(|r| r[p]).collect();
    Ok(x.iter()
        .zip(y)
        .map(|(r, v)| {
            let e = v - r.iter().zip(&beta).map(|(a, b)| a * b).sum::<f64>();
            e * e
        })
        .sum())
}
