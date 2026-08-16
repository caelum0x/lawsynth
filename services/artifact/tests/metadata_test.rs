mod support;
use lawsynth_artifact_service::{Retention, UploadOptions};

#[test]
fn metadata_records_media_type_and_retention_durably() {
    let root = support::TestRoot::new("metadata");
    let service = root.service();
    let metadata = service
        .ingest(
            b"csv".to_vec(),
            UploadOptions {
                content_type: Some("text/csv".into()),
                retention: Retention::until(99),
            },
            10,
        )
        .unwrap();
    let record = service.catalog().get(&metadata.id).unwrap();
    assert_eq!(record.content_type.as_deref(), Some("text/csv"));
    assert_eq!(record.retention, Retention::until(99));
}
