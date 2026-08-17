use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

use crate::PreprocessError;

/// Metadata sufficient to reproduce one linear resampling operation.
#[derive(Clone, Debug, PartialEq)]
pub struct ResampleReport {
    pub input_fingerprint: u64,
    pub output_fingerprint: u64,
    pub target_time: Vec<f64>,
}

/// Linearly resamples every numeric column onto an in-range target time axis.
pub fn resample_linear(
    dataset: &Dataset,
    target_time: TimeAxis,
) -> Result<Dataset, PreprocessError> {
    resample_linear_with_report(dataset, target_time).map(|(dataset, _)| dataset)
}

/// Linearly resamples a Dataset and returns deterministic provenance metadata.
pub fn resample_linear_with_report(
    dataset: &Dataset,
    target_time: TimeAxis,
) -> Result<(Dataset, ResampleReport), PreprocessError> {
    let source_time = dataset.time().values();
    if target_time.values()[0] < source_time[0]
        || target_time.values()[target_time.len() - 1] > source_time[source_time.len() - 1]
    {
        return Err(PreprocessError::ResampleOutOfBounds);
    }
    let columns = dataset
        .columns()
        .values()
        .map(|column| NumericColumn {
            id: column.id.clone(),
            values: target_time
                .values()
                .iter()
                .map(|time| interpolate(source_time, &column.values, *time))
                .collect(),
            unit: column.unit.clone(),
        })
        .collect::<Vec<_>>();
    let output = Dataset::new(target_time, columns)
        .expect("in-range interpolation preserves valid alignment");
    let report = ResampleReport {
        input_fingerprint: dataset.fingerprint(),
        output_fingerprint: output.fingerprint(),
        target_time: output.time().values().to_vec(),
    };
    Ok((output, report))
}

fn interpolate(time: &[f64], values: &[f64], target: f64) -> f64 {
    match time.binary_search_by(|candidate| candidate.total_cmp(&target)) {
        Ok(index) => values[index],
        Err(upper) => {
            let lower = upper - 1;
            let fraction = (target - time[lower]) / (time[upper] - time[lower]);
            values[lower] + fraction * (values[upper] - values[lower])
        }
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn interpolates_columns_on_a_new_axis() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 2.0]).unwrap(),
            [NumericColumn::new(Identifier::new("x").unwrap(), vec![0.0, 4.0])],
        )
        .unwrap();
        let (result, report) =
            resample_linear_with_report(&data, TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap())
                .unwrap();
        assert_eq!(result.columns()[&Identifier::new("x").unwrap()].values, vec![0.0, 2.0, 4.0]);
        assert_eq!(report.input_fingerprint, data.fingerprint());
        assert_eq!(report.output_fingerprint, result.fingerprint());
        assert_eq!(report.target_time, vec![0.0, 1.0, 2.0]);
    }
}
