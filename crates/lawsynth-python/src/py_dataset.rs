use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use std::collections::BTreeMap;

/// Constructs the validated finite Dataset used behind Python `discover` calls.
pub fn dataset_from_columns(
    time: Vec<f64>,
    columns: BTreeMap<String, Vec<f64>>,
) -> Result<Dataset, String> {
    let columns = columns
        .into_iter()
        .map(|(name, values)| {
            Ok(NumericColumn::new(
                Identifier::new(name).map_err(|error| error.to_string())?,
                values,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Dataset::new(TimeAxis::new(time).map_err(|error| error.to_string())?, columns)
        .map_err(|error| error.to_string())
}
