/// Formats a compact human-readable world summary.
pub fn world_summary(kind: &str, states: usize, variables: usize, parameters: usize) -> String {
    format!("{kind} world: {states} states, {variables} variables, {parameters} parameters\n")
}
