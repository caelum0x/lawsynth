//! Integration tests for [`SchedulerConfig`] validation and defaults.

use std::time::Duration;

use lawsynth_scheduler::{SchedulerConfig, SchedulerError};

#[test]
fn accepts_a_valid_configuration() {
    let config = SchedulerConfig::new(8, 2, Duration::from_millis(50), 8192).unwrap();
    assert_eq!(config.maximum_queued_jobs, 8);
    assert_eq!(config.maximum_attempts, 2);
    assert_eq!(config.lease_duration, Duration::from_millis(50));
    assert_eq!(config.maximum_checkpoint_bytes, 8192);
}

#[test]
fn default_is_a_valid_configuration() {
    let default = SchedulerConfig::default();
    assert_eq!(default.maximum_queued_jobs, 10_000);
    assert_eq!(default.maximum_attempts, 3);
    assert_eq!(default.lease_duration, Duration::from_secs(30));
    assert_eq!(default.maximum_checkpoint_bytes, 16 * 1024);
}

#[test]
fn rejects_zero_queue_capacity() {
    let error = SchedulerConfig::new(0, 2, Duration::from_millis(50), 8192).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidConfig(_)));
}

#[test]
fn rejects_zero_attempts() {
    let error = SchedulerConfig::new(8, 0, Duration::from_millis(50), 8192).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidConfig(_)));
}

#[test]
fn rejects_zero_lease_duration() {
    let error = SchedulerConfig::new(8, 2, Duration::ZERO, 8192).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidConfig(_)));
}

#[test]
fn rejects_a_checkpoint_ceiling_below_five_kib() {
    let error = SchedulerConfig::new(8, 2, Duration::from_millis(50), 1024).unwrap_err();
    assert!(matches!(error, SchedulerError::InvalidConfig(_)));
}
