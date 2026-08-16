mod support;

#[test]
fn health_reports_a_real_local_backend_capacity() {
    let root = support::TestRoot::new("health");
    let service = root.service();
    service.ingest(b"one".to_vec(), Default::default(), 1).unwrap();
    let report = service.health().unwrap();
    assert_eq!(report.artifact_count, 1);
    assert_eq!(report.stored_data_bytes, 3);
    assert!(report.capacity_bytes >= report.stored_data_bytes);
}
