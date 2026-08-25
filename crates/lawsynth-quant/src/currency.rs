use std::{fmt, str::FromStr};

use crate::QuantError;

/// Versioned currencies supported by the initial QR0 end-of-day boundary.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Currency {
    Chf,
    Eur,
    Gbp,
    Jpy,
    Try,
    Usd,
}

impl Currency {
    pub const ALL: [Self; 6] = [Self::Chf, Self::Eur, Self::Gbp, Self::Jpy, Self::Try, Self::Usd];

    pub const fn code(self) -> &'static str {
        match self {
            Self::Chf => "CHF",
            Self::Eur => "EUR",
            Self::Gbp => "GBP",
            Self::Jpy => "JPY",
            Self::Try => "TRY",
            Self::Usd => "USD",
        }
    }

    pub const fn minor_unit_exponent(self) -> u8 {
        match self {
            Self::Jpy => 0,
            Self::Chf | Self::Eur | Self::Gbp | Self::Try | Self::Usd => 2,
        }
    }
}

impl FromStr for Currency {
    type Err = QuantError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "CHF" => Ok(Self::Chf),
            "EUR" => Ok(Self::Eur),
            "GBP" => Ok(Self::Gbp),
            "JPY" => Ok(Self::Jpy),
            "TRY" => Ok(Self::Try),
            "USD" => Ok(Self::Usd),
            _ => Err(QuantError::UnknownCurrency(value.to_owned())),
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}
