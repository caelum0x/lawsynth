use lawsynth_core::Identifier;
use lawsynth_sim::ScheduledValue;
/// Converts Python `(time, name, value)` tuples into typed scheduled values.
pub fn scheduled_values(
    values: impl IntoIterator<Item = (f64, String, f64)>,
) -> Result<Vec<ScheduledValue>, String> {
    values
        .into_iter()
        .map(|(time, name, value)| {
            if !time.is_finite() || !value.is_finite() {
                return Err("scheduled time and value must be finite".to_owned());
            }
            Ok(ScheduledValue {
                time,
                id: Identifier::new(name).map_err(|error| error.to_string())?,
                value,
            })
        })
        .collect()
}
