use crate::{BootstrapConfig, Samples, UncertaintyError};

/// A reproducible empirical bootstrap distribution for a supplied statistic.
#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapResult {
    pub estimates: Vec<f64>,
    pub observed: f64,
}

impl BootstrapResult {
    pub fn standard_error(&self) -> Result<f64, UncertaintyError> {
        Samples::new(self.estimates.clone())?.standard_error()
    }
}

/// Resample observations with replacement and evaluate `statistic` for each replicate.
pub fn bootstrap<F>(
    samples: &Samples,
    config: BootstrapConfig,
    statistic: F,
) -> Result<BootstrapResult, UncertaintyError>
where
    F: Fn(&[f64]) -> f64,
{
    config.validate()?;
    let observed = statistic(samples.as_slice());
    if !observed.is_finite() {
        return Err(UncertaintyError::NonFiniteValue);
    }
    let mut state = config.seed;
    let mut draw = vec![0.0; samples.len()];
    let mut estimates = Vec::with_capacity(config.replicates);
    for _ in 0..config.replicates {
        for slot in &mut draw {
            *slot = samples.as_slice()[next_index(&mut state, samples.len())];
        }
        let estimate = statistic(&draw);
        if !estimate.is_finite() {
            return Err(UncertaintyError::NonFiniteValue);
        }
        estimates.push(estimate);
    }
    Ok(BootstrapResult { estimates, observed })
}

pub(crate) fn next_u64(state: &mut u64) -> u64 {
    // SplitMix64: compact, deterministic, and adequate for statistical resampling.
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    z ^ (z >> 31)
}

pub(crate) fn next_index(state: &mut u64, len: usize) -> usize {
    // Rejection sampling eliminates modulo bias.
    let zone = u64::MAX - (u64::MAX % len as u64);
    loop {
        let value = next_u64(state);
        if value < zone {
            return (value % len as u64) as usize;
        }
    }
}
