use lawsynth_runner::{CancellationToken, Checkpoint, ProcessRecord};

#[test]
fn checkpoint_records_are_verified_and_monotonic() {
    let mut record = ProcessRecord::new("work");
    record.record_checkpoint(Checkpoint::new(1, 10, b"state-one".to_vec(), 64).unwrap()).unwrap();
    assert_eq!(record.latest_checkpoint().unwrap().sequence, 1);
    assert!(
        record
            .record_checkpoint(Checkpoint::new(1, 11, b"state-two".to_vec(), 64).unwrap())
            .is_err()
    );
    let token = CancellationToken::default();
    token.cancel("operator request").unwrap();
    assert!(token.check().is_err());
}
