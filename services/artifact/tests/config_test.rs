mod support;
use lawsynth_artifact_service::{ArtifactConfig, ArtifactError, ArtifactService, UploadOptions};

#[test]
fn configuration_rejects_an_object_larger_than_a_complete_multipart_payload() {
    let mut config = ArtifactConfig::new("state");
    config.store.max_object_bytes = 10;
    config.limits.max_multipart_bytes = 9;
    assert!(config.validate().is_err());
}

#[test]
fn backend_enforces_total_capacity_before_writing_another_content_address() {
    let root = support::TestRoot::new("capacity");
    let mut config = ArtifactConfig::new(root.path());
    config.store.max_object_bytes = 512;
    config.limits.max_multipart_bytes = 512;
    config.limits.max_total_bytes = 3;
    let service = ArtifactService::open(config).unwrap();
    service.ingest(b"abc".to_vec(), UploadOptions::default(), 1).unwrap();
    assert!(matches!(
        service.ingest(b"d".to_vec(), UploadOptions::default(), 1),
        Err(ArtifactError::CapacityExceeded { requested: 1, available: 0 })
    ));
}
