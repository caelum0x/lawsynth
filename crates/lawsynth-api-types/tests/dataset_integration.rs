use lawsynth_api_types::{ColumnType, DatasetColumn, DatasetDescriptor, DatasetId, ProjectId};

#[test]
fn descriptor_rejects_ambiguous_schema_and_accepts_digest() {
    let project = ProjectId::parse("project").unwrap();
    let time = DatasetColumn::new("time", ColumnType::Float64, false).unwrap();
    let temperature = DatasetColumn::new("temperature", ColumnType::Float64, true).unwrap();
    let descriptor = DatasetDescriptor::new(
        DatasetId::parse("observations").unwrap(),
        project.clone(),
        vec![time.clone(), temperature],
        3,
        "A".repeat(64),
    )
    .unwrap();
    assert_eq!(descriptor.content_sha256, "a".repeat(64));
    assert!(
        DatasetDescriptor::new(
            DatasetId::parse("second").unwrap(),
            project,
            vec![time.clone(), time],
            2,
            "a".repeat(64)
        )
        .is_err()
    );
}
