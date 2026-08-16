use lawsynth_core::stable_hash;

use crate::{NumericColumn, TimeAxis};

/// A typed deterministic fingerprint for dataset provenance and checkpointing.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DatasetFingerprint(u64);

impl DatasetFingerprint {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// Computes a fingerprint over ordered timestamps, column identifiers, units,
/// and IEEE-754 value bits. Length delimiters prevent ambiguous concatenation.
pub(crate) fn fingerprint<'a>(
    time: &TimeAxis,
    columns: impl Iterator<Item = &'a NumericColumn>,
) -> DatasetFingerprint {
    let mut content = Vec::new();
    push_bytes(&mut content, b"lawsynth.dataset.v1");
    for value in time.values() {
        content.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    for column in columns {
        push_bytes(&mut content, column.id.as_str().as_bytes());
        match &column.unit {
            Some(unit) => push_bytes(&mut content, unit.as_bytes()),
            None => content.extend_from_slice(&u64::MAX.to_le_bytes()),
        }
        for value in &column.values {
            content.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    DatasetFingerprint(stable_hash(content))
}

fn push_bytes(target: &mut Vec<u8>, value: &[u8]) {
    target.extend_from_slice(&(value.len() as u64).to_le_bytes());
    target.extend_from_slice(value);
}
