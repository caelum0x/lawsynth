use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RewriteError {
    InvalidConfig,
    LimitExceeded,
}

impl fmt::Display for RewriteError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig => write!(formatter, "rewrite configuration is invalid"),
            Self::LimitExceeded => {
                write!(formatter, "rewrite expression exceeds configured limits")
            }
        }
    }
}
impl std::error::Error for RewriteError {}
