use std::collections::BTreeMap;

use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

use crate::PreprocessError;

/// Per-column constants needed to invert a z-score transform.
#[derive(Clone, Debug, PartialEq)]
pub struct ScaleReport {
    pub input_fingerprint: u64,
    pub output_fingerprint: u64,
    pub mean: BTreeMap<String, f64>,
    pub standard_deviation: BTreeMap<String, f64>,
    pub original_units: BTreeMap<String, Option<String>>,
}

/// Standardizes each column using population mean and standard deviation.
pub fn standardize(dataset: &Dataset) -> Result<(Dataset, ScaleReport), PreprocessError> {
    let mut mean = BTreeMap::new();
    let mut standard_deviation = BTreeMap::new();
    let mut original_units = BTreeMap::new();
    let mut columns = Vec::new();
    for column in dataset.columns().values() {
        let column_mean = column.values.iter().sum::<f64>() / column.values.len() as f64;
        let variance = column.values.iter().map(|value| (value - column_mean).powi(2)).sum::<f64>()
            / column.values.len() as f64;
        let deviation = variance.sqrt();
        if deviation <= f64::EPSILON {
            return Err(PreprocessError::ConstantColumn(column.id.to_string()));
        }
        mean.insert(column.id.to_string(), column_mean);
        standard_deviation.insert(column.id.to_string(), deviation);
        original_units.insert(column.id.to_string(), column.unit.clone());
        columns.push(NumericColumn {
            id: column.id.clone(),
            values: column.values.iter().map(|value| (value - column_mean) / deviation).collect(),
            unit: None,
        });
    }
    let scaled = Dataset::new(
        TimeAxis::new(dataset.time().values().to_vec()).expect("source time axis is valid"),
        columns,
    )
    .expect("standardization preserves valid alignment");
    let report = ScaleReport {
        input_fingerprint: dataset.fingerprint(),
        output_fingerprint: scaled.fingerprint(),
        mean,
        standard_deviation,
        original_units,
    };
    Ok((scaled, report))
}

/// Restores physical values and units from a z-score transformed Dataset.
pub fn unstandardize(dataset: &Dataset, report: &ScaleReport) -> Result<Dataset, PreprocessError> {
    let columns = dataset
        .columns()
        .values()
        .map(|column| {
            let name = column.id.to_string();
            let mean = report
                .mean
                .get(&name)
                .ok_or_else(|| PreprocessError::MissingScaleColumn(name.clone()))?;
            let deviation = report
                .standard_deviation
                .get(&name)
                .ok_or_else(|| PreprocessError::MissingScaleColumn(name.clone()))?;
            Ok(NumericColumn {
                id: column.id.clone(),
                values: column.values.iter().map(|value| value * deviation + mean).collect(),
                unit: report
                    .original_units
                    .get(&name)
                    .ok_or(PreprocessError::MissingScaleColumn(name))?
                    .clone(),
            })
        })
        .collect::<Result<Vec<_>, PreprocessError>>()?;
    Ok(Dataset::new(
        TimeAxis::new(dataset.time().values().to_vec()).expect("source time axis is valid"),
        columns,
    )
    .expect("inverse standardization preserves valid alignment"))
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn standardizes_a_nonconstant_column() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(Identifier::new("x").unwrap(), vec![1.0, 3.0]).with_unit("m")],
        )
        .unwrap();
        let (scaled, report) = standardize(&data).unwrap();
        assert_eq!(report.mean["x"], 2.0);
        assert_eq!(scaled.columns()[&Identifier::new("x").unwrap()].values, vec![-1.0, 1.0]);
        assert_eq!(scaled.columns()[&Identifier::new("x").unwrap()].unit, None);
        let restored = unstandardize(&scaled, &report).unwrap();
        assert_eq!(restored, data);
        assert_eq!(report.input_fingerprint, data.fingerprint());
        assert_eq!(report.output_fingerprint, scaled.fingerprint());
    }
}
