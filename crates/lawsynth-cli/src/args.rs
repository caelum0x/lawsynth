use crate::CliError;
use lawsynth_core::Identifier;

/// Parses a `NAME=FINITE_VALUE` command-line assignment without shell-specific behavior.
pub fn parse_assignment_text(value: &str) -> Result<(Identifier, f64), CliError> {
    let (name, number) = value
        .split_once('=')
        .ok_or(CliError::InvalidArgument("expected NAME=VALUE"))?;
    let number = number
        .parse::<f64>()
        .map_err(|_| CliError::InvalidArgument("invalid finite number"))?;
    if !number.is_finite() {
        return Err(CliError::InvalidArgument("invalid finite number"));
    }
    Ok((
        Identifier::new(name).map_err(|_| CliError::InvalidArgument("invalid identifier"))?,
        number,
    ))
}
/// Parses a nonempty comma-separated identifier list.
pub fn parse_identifier_list(value: &str) -> Result<Vec<Identifier>, CliError> {
    let values = value
        .split(',')
        .map(str::trim)
        .map(|name| {
            Identifier::new(name).map_err(|_| CliError::InvalidArgument("invalid identifier"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    if values.is_empty() {
        Err(CliError::InvalidArgument(
            "expected at least one identifier",
        ))
    } else {
        Ok(values)
    }
}
