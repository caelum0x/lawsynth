use crate::{HistogramConfig, StatsError};

/// Estimates discrete mutual information in nats using equal-width histogram
/// bins over each finite input's observed range.
pub fn histogram_mutual_information(
    left: &[f64],
    right: &[f64],
    config: HistogramConfig,
) -> Result<f64, StatsError> {
    if left.len() != right.len() {
        return Err(StatsError::LengthMismatch);
    }
    if left.len() < 2 {
        return Err(StatsError::TooFewValues);
    }
    config.validate()?;
    if left.iter().chain(right).any(|value| !value.is_finite()) {
        return Err(StatsError::NonFiniteValue);
    }
    let (left_min, left_max) = range(left);
    let (right_min, right_max) = range(right);
    if left_min == left_max || right_min == right_max {
        return Err(StatsError::ConstantValues);
    }
    let bins = config.bins;
    let mut joint = vec![0_usize; bins * bins];
    let mut left_counts = vec![0_usize; bins];
    let mut right_counts = vec![0_usize; bins];
    for (left_value, right_value) in left.iter().zip(right) {
        let left_bin = bin(*left_value, left_min, left_max, bins);
        let right_bin = bin(*right_value, right_min, right_max, bins);
        joint[left_bin * bins + right_bin] += 1;
        left_counts[left_bin] += 1;
        right_counts[right_bin] += 1;
    }
    let count = left.len() as f64;
    Ok(joint
        .iter()
        .enumerate()
        .filter(|(_, joint_count)| **joint_count > 0)
        .map(|(index, joint_count)| {
            let left_bin = index / bins;
            let right_bin = index % bins;
            let joint_probability = *joint_count as f64 / count;
            let left_probability = left_counts[left_bin] as f64 / count;
            let right_probability = right_counts[right_bin] as f64 / count;
            joint_probability * (joint_probability / (left_probability * right_probability)).ln()
        })
        .sum())
}

fn range(values: &[f64]) -> (f64, f64) {
    values.iter().copied().fold((f64::INFINITY, f64::NEG_INFINITY), |(minimum, maximum), value| {
        (minimum.min(value), maximum.max(value))
    })
}

fn bin(value: f64, minimum: f64, maximum: f64, bins: usize) -> usize {
    (((value - minimum) / (maximum - minimum) * bins as f64) as usize).min(bins - 1)
}
