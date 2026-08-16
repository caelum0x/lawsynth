mod support;
use lawsynth_artifact_service::{ArtifactError, UploadOptions};

#[test]
fn download_detects_tampering_in_the_local_object_file() {
    let root = support::TestRoot::new("download");
    let service = root.service();
    let metadata = service.ingest(b"valid payload".to_vec(), UploadOptions::default(), 1).unwrap();
    std::fs::write(root.path().join(format!("artifacts/data/{}.bin", metadata.id)), b"altered")
        .unwrap();
    assert!(matches!(service.get(&metadata.id, 2), Err(ArtifactError::ChecksumMismatch { .. })));
}
