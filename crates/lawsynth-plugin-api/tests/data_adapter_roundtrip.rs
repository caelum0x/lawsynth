use lawsynth_plugin_api::{Column, DataBatch, DataSchema, ScalarType, validate_row_group};

#[test]
fn validated_batches_preserve_row_count_and_schema() {
    let schema = DataSchema {
        columns: vec![
            Column { name: "time".into(), scalar_type: ScalarType::Float64, nullable: false },
            Column { name: "count".into(), scalar_type: ScalarType::Int64, nullable: false },
        ],
    };
    assert_eq!(
        validate_row_group(
            &schema,
            &[DataBatch::Float64(vec![0.0, 1.0]), DataBatch::Int64(vec![2, 3])],
            10
        )
        .unwrap(),
        2
    );
    assert!(
        validate_row_group(
            &schema,
            &[DataBatch::Float64(vec![f64::NAN]), DataBatch::Int64(vec![2])],
            10
        )
        .is_err()
    );
}
