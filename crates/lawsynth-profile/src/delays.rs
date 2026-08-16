use crate::{ProfileError, pearson_correlation};

/// The strongest signed lead/lag within an inspected sample window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DelayEstimate {
    /// Positive means the right series is evaluated later than the left series.
    pub lag: isize,
    pub correlation: f64,
}

/// Finds the lag with greatest absolute Pearson correlation, breaking ties by
/// lower absolute lag and then lower signed lag.
pub fn estimate_delay(
    left: &[f64],
    right: &[f64],
    max_lag: usize,
) -> Result<DelayEstimate, ProfileError> {
    if left.len() != right.len() {
        return Err(ProfileError::LengthMismatch);
    }
    if left.len() < 2 {
        return Err(ProfileError::TooFewValues);
    }
    let maximum = max_lag.min(left.len() - 2) as isize;
    let mut best = None;
    for lag in -maximum..=maximum {
        let (left_start, right_start, length) = if lag >= 0 {
            (0, lag as usize, left.len() - lag as usize)
        } else {
            ((-lag) as usize, 0, left.len() - (-lag) as usize)
        };
        let correlation = pearson_correlation(
            &left[left_start..left_start + length],
            &right[right_start..right_start + length],
        )?;
        let candidate = DelayEstimate { lag, correlation };
        if best.is_none_or(|current: DelayEstimate| {
            candidate.correlation.abs() > current.correlation.abs()
                || (candidate.correlation.abs() == current.correlation.abs()
                    && (candidate.lag.abs(), candidate.lag) < (current.lag.abs(), current.lag))
        }) {
            best = Some(candidate);
        }
    }
    Ok(best.expect("lag range includes zero"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identifies_a_positive_delay() {
        let left = [0.0, 1.0, 2.0, 3.0, 4.0];
        let right = [9.0, 0.0, 1.0, 2.0, 3.0];
        assert_eq!(estimate_delay(&left, &right, 2).unwrap().lag, 1);
    }
}
