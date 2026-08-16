use std::collections::BTreeMap;

use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

use crate::PreprocessError;

/// Fitted linear trend constants for every detrended column.
#[derive(Clone, Debug, PartialEq)]
pub struct DetrendReport {
    pub input_fingerprint: u64,
    pub output_fingerprint: u64,
    pub intercept: BTreeMap<String, f64>,
    pub slope: BTreeMap<String, f64>,
}

/// Removes a least-squares linear trend from each aligned numeric column.
pub fn detrend_linear(dataset: &Dataset) -> Result<(Dataset, DetrendReport), PreprocessError> {
    let time = dataset.time().values();
    let count = time.len() as f64;
    let time_mean = time.iter().sum::<f64>() / count;
    let time_variance = time
        .iter()
        .map(|value| (value - time_mean).powi(2))
        .sum::<f64>();
    if time_variance <= f64::EPSILON {
        return Err(PreprocessError::InvalidTargetTime);
    }
    let mut intercept = BTreeMap::new();
    let mut slope = BTreeMap::new();
    let columns = dataset
        .columns()
        .values()
        .map(|column| {
            let value_mean = column.values.iter().sum::<f64>() / count;
            let fitted_slope = time
                .iter()
                .zip(&column.values)
                .map(|(time, value)| (time - time_mean) * (value - value_mean))
                .sum::<f64>()
                / time_variance;
            let fitted_intercept = value_mean - fitted_slope * time_mean;
            intercept.insert(column.id.to_string(), fitted_intercept);
            slope.insert(column.id.to_string(), fitted_slope);
            NumericColumn {
                id: column.id.clone(),
                values: time
                    .iter()
                    .zip(&column.values)
                    .map(|(time, value)| value - (fitted_intercept + fitted_slope * time))
                    .collect(),
                unit: column.unit.clone(),
            }
        })
        .collect::<Vec<_>>();
    let output = Dataset::new(
        TimeAxis::new(time.to_vec()).expect("source time axis is valid"),
        columns,
    )
    .expect("detrending preserves valid aligned data");
    let report = DetrendReport {
        input_fingerprint: dataset.fingerprint(),
        output_fingerprint: output.fingerprint(),
        intercept,
        slope,
    };
    Ok((output, report))
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::NumericColumn;

    use super::*;

    #[test]
    fn removes_a_linear_trend_and_records_coefficients() {
        let dataset = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [NumericColumn::new(
                Identifier::new("x").unwrap(),
                vec![2.0, 5.0, 8.0],
            )],
        )
        .unwrap();
        let (detrended, report) = detrend_linear(&dataset).unwrap();
        assert_eq!(report.intercept["x"], 2.0);
        assert_eq!(report.slope["x"], 3.0);
        assert!(
            detrended.columns()[&Identifier::new("x").unwrap()]
                .values
                .iter()
                .all(|value| value.abs() < 1e-12)
        );
        assert_eq!(report.output_fingerprint, detrended.fingerprint());
    }
}
