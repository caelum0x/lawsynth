use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

use crate::PreprocessError;

/// Metadata sufficient to reproduce the initial smoothing transform.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreprocessReport {
    pub input_fingerprint: u64,
    pub output_fingerprint: u64,
    pub moving_average_radius: usize,
}

/// Applies a centered moving average while retaining endpoint windows.
pub fn moving_average(
    dataset: &Dataset,
    radius: usize,
) -> Result<(Dataset, PreprocessReport), PreprocessError> {
    if radius == 0 {
        return Err(PreprocessError::ZeroRadius);
    }
    let columns = dataset
        .columns()
        .values()
        .map(|column| {
            let values = (0..column.values.len())
                .map(|index| {
                    let start = index.saturating_sub(radius);
                    let end = (index + radius + 1).min(column.values.len());
                    column.values[start..end].iter().sum::<f64>() / (end - start) as f64
                })
                .collect();
            NumericColumn { id: column.id.clone(), values, unit: column.unit.clone() }
        })
        .collect::<Vec<_>>();
    let output = Dataset::new(
        TimeAxis::new(dataset.time().values().to_vec()).expect("source time axis is valid"),
        columns,
    )
    .expect("smoothing preserves finite aligned data");
    let report = PreprocessReport {
        input_fingerprint: dataset.fingerprint(),
        output_fingerprint: output.fingerprint(),
        moving_average_radius: radius,
    };
    Ok((output, report))
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn smooths_with_short_endpoint_windows_and_records_provenance() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [NumericColumn::new(Identifier::new("x").unwrap(), vec![0.0, 3.0, 0.0])],
        )
        .unwrap();
        let (smoothed, report) = moving_average(&data, 1).unwrap();
        assert_eq!(smoothed.columns()[&Identifier::new("x").unwrap()].values, vec![1.5, 1.0, 1.5]);
        assert_eq!(report.input_fingerprint, data.fingerprint());
        assert_eq!(report.output_fingerprint, smoothed.fingerprint());
    }
}
