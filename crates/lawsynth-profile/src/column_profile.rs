use crate::ProfileError;

/// Stable population statistics for a finite numeric column.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnProfile {
    pub minimum: f64,
    pub maximum: f64,
    pub mean: f64,
    pub variance: f64,
}

impl ColumnProfile {
    pub fn from_values(values: &[f64]) -> Result<Self, ProfileError> {
        let Some((&first, _)) = values.split_first() else {
            return Err(ProfileError::EmptyColumn);
        };
        let mut minimum = first;
        let mut maximum = first;
        let mut mean = 0.0;
        let mut sum_of_squares = 0.0;
        for (index, value) in values.iter().copied().enumerate() {
            minimum = minimum.min(value);
            maximum = maximum.max(value);
            let count = (index + 1) as f64;
            let delta = value - mean;
            mean += delta / count;
            sum_of_squares += delta * (value - mean);
        }
        Ok(Self {
            minimum,
            maximum,
            mean,
            variance: sum_of_squares / values.len() as f64,
        })
    }
}
