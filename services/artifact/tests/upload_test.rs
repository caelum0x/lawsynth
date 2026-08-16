mod support;
use lawsynth_artifact_service::UploadOptions;

#[test]
fn ingest_is_content_addressed_and_persists_across_service_reopen() {
    let root = support::TestRoot::new("ingest");
    let first = root.service();
    let metadata =
        first.ingest(b"inspectable bundle".to_vec(), UploadOptions::default(), 100).unwrap();
    let duplicate =
        first.ingest(b"inspectable bundle".to_vec(), UploadOptions::default(), 101).unwrap();
    assert_eq!(metadata, duplicate);
    drop(first);
    let reopened = root.service();
    assert_eq!(reopened.get(&metadata.id, 200).unwrap().bytes, b"inspectable bundle");
}
