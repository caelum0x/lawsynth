//! Broker transport seam — a documented boundary, not a client.
//!
//! The distributed process model in the architecture publishes job envelopes and
//! streams progress events over a message broker (NATS in the reference design).
//! This crate links **no** broker client and no wire codec for the typed
//! `JobEnvelope`, so this module defines only the *interface* such a transport
//! would implement, plus the single honest implementation — [`UnlinkedBroker`] —
//! that refuses every operation with [`BrokerError::NotLinked`]. It mirrors the
//! [`crate::SchedulerTransport::BrokerNotLinked`] surface so callers cannot
//! mistake the absence of a broker for a working one. Wiring a real client means
//! adding a dependency and a codec behind this trait; nothing here fakes that.

use crate::SchedulerTransport;

/// Failure surface for the broker seam.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrokerError {
    /// No broker client or job codec is linked into this build.
    NotLinked,
}

impl std::fmt::Display for BrokerError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotLinked => {
                write!(formatter, "{}", SchedulerTransport::BrokerNotLinked.reason())
            }
        }
    }
}

impl std::error::Error for BrokerError {}

/// The interface a message broker would implement to carry scheduler traffic.
///
/// The methods describe the publish/subscribe shape the distributed scheduler
/// would use. This crate provides no networked implementor; [`UnlinkedBroker`] is
/// the only one, and it is deliberately inert.
pub trait JobBroker {
    /// Publishes an opaque payload to a subject.
    fn publish(&self, subject: &str, payload: &[u8]) -> Result<(), BrokerError>;

    /// Whether a real transport backs this broker.
    fn is_linked(&self) -> bool;

    /// The transport surface this broker exposes.
    fn transport(&self) -> SchedulerTransport;
}

/// The honest default: a broker with nothing behind it.
///
/// Every publish fails with [`BrokerError::NotLinked`] and the surface reports
/// itself unavailable, exactly matching [`SchedulerTransport::BrokerNotLinked`].
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnlinkedBroker;

impl JobBroker for UnlinkedBroker {
    fn publish(&self, _subject: &str, _payload: &[u8]) -> Result<(), BrokerError> {
        Err(BrokerError::NotLinked)
    }

    fn is_linked(&self) -> bool {
        false
    }

    fn transport(&self) -> SchedulerTransport {
        SchedulerTransport::BrokerNotLinked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlinked_broker_refuses_to_publish() {
        let broker = UnlinkedBroker;
        assert_eq!(broker.publish("jobs.submit", b"payload"), Err(BrokerError::NotLinked));
    }

    #[test]
    fn unlinked_broker_reports_its_surface_honestly() {
        let broker = UnlinkedBroker;
        assert!(!broker.is_linked());
        assert_eq!(broker.transport(), SchedulerTransport::BrokerNotLinked);
        assert!(!broker.transport().is_available());
    }

    #[test]
    fn error_message_matches_the_transport_reason() {
        assert_eq!(
            BrokerError::NotLinked.to_string(),
            SchedulerTransport::BrokerNotLinked.reason()
        );
    }
}
