use std::collections::BTreeMap;

use lawsynth_core::Identifier;

use crate::{
    DataError, DatasetBatch, DatasetFingerprint, DatasetSchema, NumericColumn, TimeAxis,
    WindowConfig, fingerprint::fingerprint,
};

/// An immutable, aligned collection of finite numeric observations.
#[derive(Clone, Debug, PartialEq)]
pub struct Dataset {
    time: TimeAxis,
    columns: BTreeMap<Identifier, NumericColumn>,
}

impl Dataset {
    pub fn new(
        time: TimeAxis,
        columns: impl IntoIterator<Item = NumericColumn>,
    ) -> Result<Self, DataError> {
        let mut column_map = BTreeMap::new();
        for column in columns {
            if column.values.len() != time.len() {
                return Err(DataError::ColumnLengthMismatch {
                    column: column.id,
                    expected: time.len(),
                    actual: column.values.len(),
                });
            }
            for (index, value) in column.values.iter().copied().enumerate() {
                if !value.is_finite() {
                    return Err(DataError::NonFiniteValue {
                        column: column.id,
                        index,
                        value,
                    });
                }
            }
            if column_map
                .insert(column.id.clone(), column.clone())
                .is_some()
            {
                return Err(DataError::DuplicateColumn(column.id));
            }
        }
        if column_map.is_empty() {
            return Err(DataError::NoColumns);
        }
        Ok(Self {
            time,
            columns: column_map,
        })
    }

    pub fn time(&self) -> &TimeAxis {
        &self.time
    }

    pub fn columns(&self) -> &BTreeMap<Identifier, NumericColumn> {
        &self.columns
    }

    /// Returns the stable, lexicographically ordered numeric column schema.
    pub fn schema(&self) -> DatasetSchema {
        DatasetSchema {
            columns: self.columns.keys().cloned().collect(),
        }
    }

    /// Splits observations into deterministic, aligned, owned batches.
    pub fn batches(&self, batch_size: usize) -> Result<Vec<DatasetBatch>, DataError> {
        if batch_size == 0 {
            return Err(DataError::InvalidBatchSize);
        }
        let mut batches = Vec::new();
        for start in (0..self.time.len()).step_by(batch_size) {
            let end = (start + batch_size).min(self.time.len());
            batches.push(DatasetBatch {
                rows: start..end,
                time: self.time.values()[start..end].to_vec(),
                columns: self
                    .columns
                    .iter()
                    .map(|(id, column)| (id.clone(), column.values[start..end].to_vec()))
                    .collect(),
            });
        }
        Ok(batches)
    }

    /// Returns complete, aligned sliding windows in chronological order.
    pub fn windows(&self, config: WindowConfig) -> Result<Vec<DatasetBatch>, DataError> {
        if config.width == 0 || config.step == 0 || config.width > self.time.len() {
            return Err(DataError::InvalidWindowConfig);
        }
        Ok((0..=self.time.len() - config.width)
            .step_by(config.step)
            .map(|start| self.batch(start, start + config.width))
            .collect())
    }

    fn batch(&self, start: usize, end: usize) -> DatasetBatch {
        DatasetBatch {
            rows: start..end,
            time: self.time.values()[start..end].to_vec(),
            columns: self
                .columns
                .iter()
                .map(|(id, column)| (id.clone(), column.values[start..end].to_vec()))
                .collect(),
        }
    }

    /// A deterministic content fingerprint for reproducibility metadata.
    pub fn fingerprint(&self) -> u64 {
        self.content_fingerprint().value()
    }

    /// A typed content fingerprint that includes schema unit metadata.
    pub fn content_fingerprint(&self) -> DatasetFingerprint {
        fingerprint(&self.time, self.columns.values())
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn preserves_sorted_columns_and_a_stable_fingerprint() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [
                NumericColumn::new(id("y"), vec![0.0, 1.0, 4.0]),
                NumericColumn::new(id("x"), vec![0.0, 1.0, 2.0]),
            ],
        )
        .unwrap();
        assert_eq!(data.columns().keys().next().unwrap().as_str(), "x");
        assert_eq!(data.fingerprint(), data.fingerprint());
    }

    #[test]
    fn rejects_misaligned_columns() {
        let result = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0]).unwrap(),
            [NumericColumn::new(id("x"), vec![0.0])],
        );
        assert!(matches!(
            result,
            Err(DataError::ColumnLengthMismatch { .. })
        ));
    }

    #[test]
    fn exposes_ordered_schema_and_aligned_batches() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
            [
                NumericColumn::new(id("y"), vec![0.0, 1.0, 4.0]),
                NumericColumn::new(id("x"), vec![3.0, 2.0, 1.0]),
            ],
        )
        .unwrap();
        assert_eq!(
            data.schema().columns,
            vec![Identifier::new("x").unwrap(), Identifier::new("y").unwrap()]
        );
        let batches = data.batches(2).unwrap();
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].rows, 0..2);
        assert_eq!(batches[1].time, vec![2.0]);
        assert_eq!(batches[0].columns[&id("x")], vec![3.0, 2.0]);
    }

    #[test]
    fn exposes_complete_sliding_windows() {
        let data = Dataset::new(
            TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0]).unwrap(),
            [NumericColumn::new(id("x"), vec![10.0, 11.0, 12.0, 13.0])],
        )
        .unwrap();
        let windows = data.windows(WindowConfig::new(3, 1)).unwrap();
        assert_eq!(windows.len(), 2);
        assert_eq!(windows[0].rows, 0..3);
        assert_eq!(windows[1].rows, 1..4);
        assert_eq!(windows[1].columns[&id("x")], vec![11.0, 12.0, 13.0]);
        assert_eq!(
            data.windows(WindowConfig::new(5, 1)),
            Err(DataError::InvalidWindowConfig)
        );
    }
}
