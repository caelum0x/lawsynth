use crate::{CliError, args::parse_assignment_text};
use lawsynth_core::Identifier;
/// Parses `TIME:NAME=VALUE` scheduled intervention syntax.
pub fn parse_scheduled_assignment(value: &str) -> Result<(f64, Identifier, f64), CliError> {
    let (time, assignment) = value
        .split_once(':')
        .ok_or(CliError::InvalidArgument("expected TIME:NAME=VALUE"))?;
    let time = time
        .parse::<f64>()
        .map_err(|_| CliError::InvalidArgument("invalid intervention time"))?;
    if !time.is_finite() {
        return Err(CliError::InvalidArgument("invalid intervention time"));
    }
    let (id, value) = parse_assignment_text(assignment)?;
    Ok((time, id, value))
}
