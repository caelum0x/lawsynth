use lawsynth_sim::Trajectory;
use std::collections::BTreeMap;
/// Converts typed trajectory identifiers into Python-friendly string keys.
pub fn trajectory_values(trajectory: &Trajectory) -> BTreeMap<String, Vec<f64>> {
    trajectory.values.iter().map(|(id, values)| (id.to_string(), values.clone())).collect()
}
