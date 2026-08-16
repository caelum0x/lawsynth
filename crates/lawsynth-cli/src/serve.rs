use crate::CliError;
/// The CLI intentionally has no daemon mode in the initial offline build.
/// Returning an explicit error prevents callers from assuming a server exists.
pub fn unavailable() -> Result<(), CliError> {
    Err(CliError::Unsupported(
        "serve mode is not compiled into this distribution",
    ))
}
