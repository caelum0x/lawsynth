/// Validation policy applied while constructing executable World IR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldConfig {
    /// Reject expressions that read identifiers not declared as variables or parameters.
    pub validate_expression_symbols: bool,
    /// Check declared physical dimensions when units are available.
    pub validate_units: bool,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            validate_expression_symbols: true,
            validate_units: true,
        }
    }
}
