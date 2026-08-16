mod support;
use lawsynth_artifact_service::{Retention, UploadOptions};

#[test]
fn collection_deletes_expired_bytes_and_preserves_live_data() {
    let root = support::TestRoot::new("gc");
    let service = root.service();
    let expired = service
        .ingest(
            b"expired".to_vec(),
            UploadOptions { content_type: None, retention: Retention::until(4) },
            1,
        )
        .unwrap();
    let live = service.ingest(b"live".to_vec(), UploadOptions::default(), 1).unwrap();
    let report = service.collect_garbage(4, false).unwrap();
    assert_eq!(report.deleted, vec![expired.id]);
    assert_eq!(service.get(&live.id, 4).unwrap().bytes, b"live");
}
