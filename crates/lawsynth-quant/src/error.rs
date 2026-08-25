use std::fmt;

use lawsynth_core::IdentifierError;

use crate::Currency;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QuantError {
    ArithmeticOverflow,
    CurrencyMismatch { left: Currency, right: Currency },
    InstrumentMismatch { left: String, right: String },
    InstrumentTooLong { bytes: usize },
    InvalidEncoding(&'static str),
    InvalidInstrument(IdentifierError),
    UnknownCurrency(String),
}

impl fmt::Display for QuantError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow => formatter.write_str("money arithmetic overflow"),
            Self::CurrencyMismatch { left, right } => {
                write!(formatter, "currency mismatch: {left} and {right}")
            }
            Self::InstrumentMismatch { left, right } => {
                write!(formatter, "instrument mismatch: {left} and {right}")
            }
            Self::InstrumentTooLong { bytes } => {
                write!(formatter, "instrument identifier is {bytes} bytes; maximum is 65535")
            }
            Self::InvalidEncoding(reason) => write!(formatter, "invalid quant encoding: {reason}"),
            Self::InvalidInstrument(error) => {
                write!(formatter, "invalid instrument identifier: {error}")
            }
            Self::UnknownCurrency(code) => write!(formatter, "unsupported currency code: {code}"),
        }
    }
}

impl std::error::Error for QuantError {}

impl From<IdentifierError> for QuantError {
    fn from(value: IdentifierError) -> Self {
        Self::InvalidInstrument(value)
    }
}
