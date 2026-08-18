use std::collections::BTreeMap;

use lawsynth_core::Identifier;

/// Offsets a state by a derivative vector without mutating either source map.
pub(crate) fn offset_state(
    state: &BTreeMap<Identifier, f64>,
    derivative: &BTreeMap<Identifier, f64>,
    scale: f64,
) -> BTreeMap<Identifier, f64> {
    state.iter().map(|(id, value)| (id.clone(), value + derivative[id] * scale)).collect()
}
