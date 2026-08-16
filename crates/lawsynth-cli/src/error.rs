use std::fmt;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CliError {
    InvalidArgument(&'static str),
    Unsupported(&'static str),
}
impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(message) => write!(f, "invalid argument: {message}"),
            Self::Unsupported(message) => write!(f, "unsupported: {message}"),
        }
    }
}
impl std::error::Error for CliError {}
