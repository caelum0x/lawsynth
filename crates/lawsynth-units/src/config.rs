/// Parsing policy for quantity units supplied by external data sources.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnitConfig {
    pub allow_custom_units: bool,
}
impl Default for UnitConfig {
    fn default() -> Self {
        Self { allow_custom_units: true }
    }
}
