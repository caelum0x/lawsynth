//! Expired-lease recovery decisions.
//!
//! A leased job whose lease has expired without a heartbeat — or whose hard
//! deadline elapsed while leased — must be returned to a schedulable state. This
//! module holds the pure decision the scheduler applies during
//! [`crate::Scheduler::recover_expired`]: whether to requeue the job for another
//! attempt or dead-letter it, and with what human-readable reason. Per the
//! architecture, "worker loss returns the job to schedulable state after lease
//! expiry" while an already-elapsed deadline can only be dead-lettered.

/// The outcome of recovering one expired lease.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryOutcome {
    /// Return the job to the queue for another attempt.
    Requeue,
    /// Terminate the job with the given reason.
    DeadLetter { reason: String },
}

/// Reason recorded when a queued job's deadline elapses before assignment.
pub const QUEUED_EXPIRY_REASON: &str = "job deadline elapsed before assignment";

/// Decides how to recover an expired lease.
///
/// A job is requeued only if attempts remain *and* its deadline has not already
/// elapsed. Otherwise it is dead-lettered, with the reason distinguishing a
/// blown deadline from a silent worker.
pub fn on_lease_expiry(
    attempt: u32,
    maximum_attempts: u32,
    deadline_elapsed: bool,
) -> RecoveryOutcome {
    if attempt < maximum_attempts && !deadline_elapsed {
        RecoveryOutcome::Requeue
    } else if deadline_elapsed {
        RecoveryOutcome::DeadLetter { reason: "job deadline elapsed while leased".into() }
    } else {
        RecoveryOutcome::DeadLetter { reason: "worker lease expired without a heartbeat".into() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requeues_while_attempts_remain_and_deadline_is_live() {
        assert_eq!(on_lease_expiry(1, 3, false), RecoveryOutcome::Requeue);
    }

    #[test]
    fn dead_letters_when_attempts_are_exhausted() {
        let outcome = on_lease_expiry(3, 3, false);
        let RecoveryOutcome::DeadLetter { reason } = outcome else {
            panic!("expected dead letter");
        };
        assert!(reason.contains("without a heartbeat"));
    }

    #[test]
    fn dead_letters_when_deadline_elapsed_even_with_attempts_left() {
        let outcome = on_lease_expiry(1, 3, true);
        let RecoveryOutcome::DeadLetter { reason } = outcome else {
            panic!("expected dead letter");
        };
        assert!(reason.contains("while leased"));
    }
}
