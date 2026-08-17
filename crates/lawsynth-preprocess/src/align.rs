use lawsynth_data::TimeAxis;

use crate::PreprocessError;

/// Aligns an independently sampled finite source series onto a target axis by
/// linear interpolation. It does not extrapolate beyond observed coverage.
pub fn align_series_linear(
    source_time: &[f64],
    source_values: &[f64],
    target_time: &TimeAxis,
) -> Result<Vec<f64>, PreprocessError> {
    if source_time.len() != source_values.len() {
        return Err(PreprocessError::AlignmentLengthMismatch);
    }
    if source_time.len() < 2
        || source_time.iter().any(|value| !value.is_finite())
        || source_values.iter().any(|value| !value.is_finite())
        || source_time.windows(2).any(|pair| pair[1] <= pair[0])
    {
        return Err(PreprocessError::InvalidAlignmentSource);
    }
    if target_time.values()[0] < source_time[0]
        || target_time.values()[target_time.len() - 1] > source_time[source_time.len() - 1]
    {
        return Err(PreprocessError::ResampleOutOfBounds);
    }
    target_time
        .values()
        .iter()
        .map(|target| match source_time.binary_search_by(|time| time.total_cmp(target)) {
            Ok(index) => Ok(source_values[index]),
            Err(upper) => {
                let lower = upper - 1;
                let fraction =
                    (target - source_time[lower]) / (source_time[upper] - source_time[lower]);
                Ok(source_values[lower] + fraction * (source_values[upper] - source_values[lower]))
            }
        })
        .collect()
}
