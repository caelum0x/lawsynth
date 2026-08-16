use lawsynth_core::Identifier;
use std::collections::BTreeMap;

/// Converts Python-style string-keyed finite values to typed World-IR ids.
pub fn identifier_values(
    values: BTreeMap<String, f64>,
) -> Result<BTreeMap<Identifier, f64>, String> {
    values
        .into_iter()
        .map(|(name, value)| {
            if !value.is_finite() {
                return Err(format!("value for '{name}' must be finite"));
            }
            Ok((
                Identifier::new(name).map_err(|error| error.to_string())?,
                value,
            ))
        })
        .collect()
}
