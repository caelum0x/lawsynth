use lawsynth_core::Identifier;
/// Converts Python state names to a nonempty typed discovery state plan.
pub fn state_identifiers(
    values: impl IntoIterator<Item = String>,
) -> Result<Vec<Identifier>, String> {
    let values = values
        .into_iter()
        .map(|value| Identifier::new(value).map_err(|error| error.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() { Err("at least one state is required".to_owned()) } else { Ok(values) }
}
