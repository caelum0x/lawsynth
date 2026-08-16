use crate::{RegimeError, Result};
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BocpdConfig {
    pub hazard: f64,
    pub prior_mean: f64,
    pub observation_variance: f64,
    pub prior_precision: f64,
}
impl Default for BocpdConfig {
    fn default() -> Self {
        Self {
            hazard: 0.02,
            prior_mean: 0.0,
            observation_variance: 1.0,
            prior_precision: 1e-3,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BocpdPoint {
    pub index: usize,
    pub change_probability: f64,
    pub most_likely_run_length: usize,
}
pub fn bocpd(data: &[f64], config: BocpdConfig) -> Result<Vec<BocpdPoint>> {
    if data.is_empty() {
        return Err(RegimeError::EmptySeries);
    }
    if !(0.0..=1.0).contains(&config.hazard)
        || config.hazard == 0.0
        || !config.observation_variance.is_finite()
        || config.observation_variance <= 0.0
        || config.prior_precision <= 0.0
    {
        return Err(RegimeError::InvalidParameter("bocpd config"));
    }
    let mut probs = vec![1.0];
    let mut counts = vec![0usize];
    let mut sums = vec![0.0];
    let mut out = Vec::with_capacity(data.len());
    for (index, &x) in data.iter().enumerate() {
        if !x.is_finite() {
            return Err(RegimeError::NonFiniteObservation { index });
        }
        let mut growth = Vec::with_capacity(probs.len());
        let mut change = 0.0;
        for i in 0..probs.len() {
            let precision = config.prior_precision + counts[i] as f64 / config.observation_variance;
            let mean = (config.prior_precision * config.prior_mean
                + sums[i] / config.observation_variance)
                / precision;
            let predictive_var = config.observation_variance + 1.0 / precision;
            let density = (-0.5 * (x - mean).powi(2) / predictive_var).exp()
                / (2.0 * std::f64::consts::PI * predictive_var).sqrt();
            change += probs[i] * config.hazard * density;
            growth.push(probs[i] * (1.0 - config.hazard) * density);
        }
        let normalizer = change + growth.iter().sum::<f64>();
        if normalizer == 0.0 {
            return Err(RegimeError::ImpossibleObservation { index });
        }
        let cp = change / normalizer;
        let mut new_probs = Vec::with_capacity(growth.len() + 1);
        new_probs.push(cp);
        new_probs.extend(growth.into_iter().map(|p| p / normalizer));
        let best = new_probs
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        out.push(BocpdPoint {
            index,
            change_probability: cp,
            most_likely_run_length: best,
        });
        let mut new_counts = Vec::with_capacity(counts.len() + 1);
        let mut new_sums = Vec::with_capacity(sums.len() + 1);
        new_counts.push(0);
        new_sums.push(0.0);
        new_counts.extend(counts.iter().map(|c| c + 1));
        new_sums.extend(sums.iter().map(|s| s + x));
        probs = new_probs;
        counts = new_counts;
        sums = new_sums;
    }
    Ok(out)
}
