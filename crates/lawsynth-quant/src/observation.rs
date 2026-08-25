use lawsynth_core::{Identifier, stable_hash};

use crate::QuantError;

const OBSERVATION_MAGIC: &[u8; 5] = b"LSQO1";
const FIXED_OBSERVATION_BYTES: usize = 5 + 2 + 8 + 4;

/// A UTC instant expressed as signed milliseconds from the Unix epoch.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UtcTimestamp(i64);

impl UtcTimestamp {
    pub const fn from_unix_millis(value: i64) -> Self {
        Self(value)
    }

    pub const fn unix_millis(self) -> i64 {
        self.0
    }
}

/// Stable identity and ordering for one observed instrument event.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservationKey {
    instrument: Identifier,
    timestamp: UtcTimestamp,
    sequence: u32,
}

impl ObservationKey {
    pub fn new(
        instrument: impl Into<String>,
        timestamp: UtcTimestamp,
        sequence: u32,
    ) -> Result<Self, QuantError> {
        let instrument = Identifier::new(instrument)?;
        if instrument.as_str().len() > usize::from(u16::MAX) {
            return Err(QuantError::InstrumentTooLong { bytes: instrument.as_str().len() });
        }
        Ok(Self { instrument, timestamp, sequence })
    }

    pub fn instrument(&self) -> &Identifier {
        &self.instrument
    }

    pub const fn timestamp(&self) -> UtcTimestamp {
        self.timestamp
    }

    pub const fn sequence(&self) -> u32 {
        self.sequence
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        let instrument = self.instrument.as_str().as_bytes();
        let mut bytes = Vec::with_capacity(FIXED_OBSERVATION_BYTES + instrument.len());
        bytes.extend_from_slice(OBSERVATION_MAGIC);
        bytes.extend_from_slice(&(instrument.len() as u16).to_be_bytes());
        bytes.extend_from_slice(instrument);
        bytes.extend_from_slice(&self.timestamp.0.to_be_bytes());
        bytes.extend_from_slice(&self.sequence.to_be_bytes());
        bytes
    }

    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, QuantError> {
        if bytes.len() < FIXED_OBSERVATION_BYTES + 1 {
            return Err(QuantError::InvalidEncoding("observation encoding is truncated"));
        }
        if &bytes[..5] != OBSERVATION_MAGIC {
            return Err(QuantError::InvalidEncoding("unsupported observation encoding version"));
        }
        let instrument_len = usize::from(u16::from_be_bytes([bytes[5], bytes[6]]));
        let expected_len = FIXED_OBSERVATION_BYTES
            .checked_add(instrument_len)
            .ok_or(QuantError::InvalidEncoding("observation length overflow"))?;
        if bytes.len() != expected_len {
            return Err(QuantError::InvalidEncoding(
                "observation length does not match instrument length",
            ));
        }
        let instrument_end = 7 + instrument_len;
        let instrument = std::str::from_utf8(&bytes[7..instrument_end])
            .map_err(|_| QuantError::InvalidEncoding("instrument identifier is not UTF-8"))?;
        let timestamp = i64::from_be_bytes(
            bytes[instrument_end..instrument_end + 8]
                .try_into()
                .map_err(|_| QuantError::InvalidEncoding("invalid observation timestamp"))?,
        );
        let sequence = u32::from_be_bytes(
            bytes[instrument_end + 8..]
                .try_into()
                .map_err(|_| QuantError::InvalidEncoding("invalid observation sequence"))?,
        );
        Self::new(instrument, UtcTimestamp(timestamp), sequence)
    }

    pub fn stable_fingerprint(&self) -> u64 {
        stable_hash(self.canonical_bytes())
    }
}
