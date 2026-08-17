use crate::ProfileError;

/// Order-statistic summary suitable for robust data-quality diagnostics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DistributionProfile {
    pub minimum: f64,
    pub first_quartile: f64,
    pub median: f64,
    pub third_quartile: f64,
    pub maximum: f64,
}

/// Computes deterministic linearly interpolated quartiles for finite values.
pub fn distribution(values: &[f64]) -> Result<DistributionProfile, ProfileError> {
    if values.is_empty() {
        return Err(ProfileError::EmptyColumn);
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(ProfileError::NonFiniteValues);
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    Ok(DistributionProfile {
        minimum: sorted[0],
        first_quartile: quantile(&sorted, 0.25),
        median: quantile(&sorted, 0.5),
        third_quartile: quantile(&sorted, 0.75),
        maximum: sorted[sorted.len() - 1],
    })
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
    fn reports_interpolated_quartiles_in_value_order() {
        assert_eq!(
            distribution(&[5.0, 1.0, 4.0, 2.0, 3.0]).unwrap(),
            DistributionProfile {
                minimum: 1.0,
                first_quartile: 2.0,
                median: 3.0,
                third_quartile: 4.0,
                maximum: 5.0,
            }
        );
    }

    #[test]
    fn rejects_non_finite_values() {
        assert_eq!(distribution(&[1.0, f64::NAN]), Err(ProfileError::NonFiniteValues));
    }
}
