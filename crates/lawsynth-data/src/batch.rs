use std::{collections::BTreeMap, ops::Range};

use lawsynth_core::Identifier;

/// An owned, aligned row range from a Dataset.
#[derive(Clone, Debug, PartialEq)]
pub struct DatasetBatch {
    pub rows: Range<usize>,
    pub time: Vec<f64>,
    pub columns: BTreeMap<Identifier, Vec<f64>>,
}

impl DatasetBatch {
    pub fn len(&self) -> usize {
        self.time.len()
    }

    pub fn is_empty(&self) -> bool {
        self.time.is_empty()
    }
}
