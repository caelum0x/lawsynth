use std::collections::BTreeMap;

use lawsynth_core::Identifier;
use lawsynth_data::Dataset;

use crate::{
    ColumnProfile, ColumnQuality, MissingnessProfile, ProfileConfig, ProfileError, TimeProfile,
    profile_f64_missingness, quality_flags,
};

/// Reproducible input metadata captured before discovery begins.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetProfile {
    pub fingerprint: u64,
    pub samples: usize,
    pub time: TimeProfile,
    pub columns: BTreeMap<Identifier, ColumnProfile>,
    pub quality: BTreeMap<Identifier, ColumnQuality>,
    pub missingness: BTreeMap<Identifier, MissingnessProfile>,
}

pub fn profile(dataset: &Dataset) -> Result<DatasetProfile, ProfileError> {
    profile_with_config(dataset, ProfileConfig::default())
}

/// Profiles a finite dataset while retaining explicit configuration provenance.
pub fn profile_with_config(
    dataset: &Dataset,
    config: ProfileConfig,
) -> Result<DatasetProfile, ProfileError> {
    let config = config.validate()?;
    let columns = dataset
        .columns()
        .iter()
        .map(|(id, column)| Ok((id.clone(), ColumnProfile::from_values(&column.values)?)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let quality = dataset
        .columns()
        .iter()
        .map(|(id, column)| Ok((id.clone(), quality_flags(&column.values)?)))
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let missingness = dataset
        .columns()
        .iter()
        .map(|(id, column)| Ok((id.clone(), profile_f64_missingness(&column.values))))
        .collect::<Result<BTreeMap<_, _>, ProfileError>>()?;
    Ok(DatasetProfile {
        fingerprint: dataset.fingerprint(),
        samples: dataset.time().len(),
        time: TimeProfile::from_time_axis_with_tolerance(
            dataset.time(),
            config.regularity_tolerance,
        ),
        columns,
        quality,
        missingness,
    })
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;
    use lawsynth_data::{Dataset, NumericColumn, TimeAxis};

    use super::*;

    #[test]
    fn profiles_sampling_and_population_moments() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 0.5, 1.0]).unwrap(),
            [NumericColumn::new(
                Identifier::new("x").unwrap(),
                vec![1.0, 2.0, 3.0],
            )],
        )
        .unwrap();
        let result = profile(&data).unwrap();
        let column = &result.columns[&Identifier::new("x").unwrap()];
        assert_eq!(result.samples, 3);
        assert!(result.time.is_regular);
        assert_eq!(column.minimum, 1.0);
        assert_eq!(column.maximum, 3.0);
        assert_eq!(column.mean, 2.0);
        assert!((column.variance - 2.0 / 3.0).abs() < 1e-12);
        assert!(!result.quality[&Identifier::new("x").unwrap()].is_constant);
        assert_eq!(
            result.missingness[&Identifier::new("x").unwrap()].missing,
            0
        );
    }
}
