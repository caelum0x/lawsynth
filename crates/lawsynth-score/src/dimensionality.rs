use crate::ScoreError;

/// Likelihood-based criteria for comparing models fitted to the same target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InformationCriteria {
    pub aic: f64,
    pub aicc: Option<f64>,
    pub bic: f64,
}

/// Computes Gaussian-residual AIC, finite-sample AICc when defined, and BIC.
pub fn information_criteria(
    observations: usize,
    parameters: usize,
    residual_sum_squares: f64,
) -> Result<InformationCriteria, ScoreError> {
    if observations == 0 || !residual_sum_squares.is_finite() || residual_sum_squares < 0.0 {
        return Err(ScoreError::InvalidDegreesOfFreedom);
    }
    let n = observations as f64;
    // Clamp the variance estimate only for the log likelihood: a perfect fit
    // has an unbounded ideal Gaussian likelihood, which cannot be represented
    // as a finite ranking value.
    let variance = (residual_sum_squares / n).max(f64::MIN_POSITIVE);
    let aic = n * variance.ln() + 2.0 * parameters as f64;
    let denominator = observations as isize - parameters as isize - 1;
    let aicc = (denominator > 0)
        .then(|| aic + (2 * parameters * (parameters + 1)) as f64 / denominator as f64);
    Ok(InformationCriteria { aic, aicc, bic: n * variance.ln() + parameters as f64 * n.ln() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finite_sample_correction_requires_available_degrees_of_freedom() {
        let criteria = information_criteria(10, 2, 5.0).unwrap();
        assert!(criteria.aicc.unwrap() > criteria.aic);
        assert!(information_criteria(3, 2, 1.0).unwrap().aicc.is_none());
    }
}
