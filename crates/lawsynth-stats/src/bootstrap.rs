use lawsynth_core::Seed;

use crate::StatsError;

/// Configuration for deterministic moving-block bootstrap resampling.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BootstrapConfig {
    pub replicates: usize,
    pub block_size: usize,
    pub seed: u64,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self { replicates: 100, block_size: 8, seed: 0 }
    }
}

/// A two-sided percentile confidence interval.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PercentileInterval {
    pub lower: f64,
    pub upper: f64,
}

/// Produces reproducible moving-block samples that preserve local time-series structure.
pub fn bootstrap_indices(
    observations: usize,
    config: &BootstrapConfig,
) -> Result<Vec<Vec<usize>>, StatsError> {
    if observations == 0 {
        return Err(StatsError::EmptyInput);
    }
    if config.replicates == 0 || config.block_size == 0 {
        return Err(StatsError::InvalidBootstrapConfig);
    }
    let mut rng = Seed::new(config.seed).rng();
    let mut samples = Vec::with_capacity(config.replicates);
    for _ in 0..config.replicates {
        let mut sample = Vec::with_capacity(observations);
        while sample.len() < observations {
            let start = (rng.next_u64() as usize) % observations;
            for offset in 0..config.block_size {
                if sample.len() == observations {
                    break;
                }
                sample.push((start + offset) % observations);
            }
        }
        samples.push(sample);
    }
    Ok(samples)
}

/// Computes a deterministic linear-interpolated percentile interval.
pub fn percentile_interval(
    values: &[f64],
    confidence: f64,
) -> Result<PercentileInterval, StatsError> {
    if values.is_empty() {
        return Err(StatsError::EmptyInput);
    }
    if !(0.0..=1.0).contains(&confidence) || !confidence.is_finite() {
        return Err(StatsError::InvalidConfidence);
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let tail = (1.0 - confidence) / 2.0;
    Ok(PercentileInterval { lower: quantile(&sorted, tail), upper: quantile(&sorted, 1.0 - tail) })
}

fn quantile(values: &[f64], probability: f64) -> f64 {
    let position = probability * (values.len() - 1) as f64;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    values[lower] + (values[upper] - values[lower]) * (position - lower as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_bootstrap_is_reproducible_and_aligned() {
        let config = BootstrapConfig { replicates: 3, block_size: 2, seed: 42 };
        let first = bootstrap_indices(7, &config).unwrap();
        assert_eq!(first, bootstrap_indices(7, &config).unwrap());
        assert!(
            first.iter().all(|sample| sample.len() == 7 && sample.iter().all(|index| *index < 7))
        );
    }

    #[test]
    fn computes_a_percentile_interval() {
        assert_eq!(
            percentile_interval(&[1.0, 2.0, 3.0, 4.0, 5.0], 0.5).unwrap(),
            PercentileInterval { lower: 2.0, upper: 4.0 }
        );
    }
}
