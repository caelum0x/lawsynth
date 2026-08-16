mod support;
use lawsynth_artifact_service::{ArtifactError, UploadOptions};

#[test]
fn incomplete_multipart_upload_cannot_publish_and_can_be_completed_after_repair() {
    let root = support::TestRoot::new("multipart");
    let service = root.service();
    let upload = service.begin_multipart(UploadOptions::default()).unwrap();
    service.add_multipart_part(&upload, 1, b"Law".to_vec()).unwrap();
    service.add_multipart_part(&upload, 3, b"th".to_vec()).unwrap();
    assert!(matches!(service.complete_multipart(&upload, 1), Err(ArtifactError::InvalidUpload(_))));
    service.add_multipart_part(&upload, 2, b"Syn".to_vec()).unwrap();
    let metadata = service.complete_multipart(&upload, 2).unwrap();
    assert_eq!(service.get(&metadata.id, 2).unwrap().bytes, b"LawSynth");
}
