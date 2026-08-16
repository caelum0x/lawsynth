mod support;
use lawsynth_artifact_service::{ArtifactError, Retention, UploadOptions};

#[test]
fn expired_artifacts_are_not_readable_before_collection() {
    let root = support::TestRoot::new("retention");
    let service = root.service();
    let metadata = service
        .ingest(
            b"old".to_vec(),
            UploadOptions { content_type: None, retention: Retention::until(5) },
            1,
        )
        .unwrap();
    assert!(matches!(service.get(&metadata.id, 5), Err(ArtifactError::Expired(_))));
    assert_eq!(service.collect_garbage(5, true).unwrap().deleted, vec![metadata.id.clone()]);
}
