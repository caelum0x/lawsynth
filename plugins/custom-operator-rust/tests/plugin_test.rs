use lawsynth_custom_operator::LinearOperator;
use lawsynth_plugin_api::{
    AlgorithmPlugin, AlgorithmRequest, Column, DataBatch, DataSchema, PluginError, ScalarType,
};

fn schema() -> DataSchema {
    DataSchema {
        columns: vec![
            Column {
                name: "x".into(),
                scalar_type: ScalarType::Float64,
                nullable: false,
            },
            Column {
                name: "y".into(),
                scalar_type: ScalarType::Float64,
                nullable: false,
            },
        ],
    }
}

fn request(target: &str) -> AlgorithmRequest {
    AlgorithmRequest {
        schema: schema(),
        columns: vec![
            DataBatch::Float64(vec![0.0, 1.0, 2.0, 3.0]),
            DataBatch::Float64(vec![1.0, 3.0, 5.0, 7.0]),
        ],
        target: target.into(),
    }
}

#[test]
fn discovers_linear_relationship() {
    let response = LinearOperator::default().discover(request("y")).unwrap();
    // y = 2x + 1, so the best predictor is `x` and the fit is essentially exact.
    assert!(response.equation.contains("* x"));
    assert!(response.score.is_finite());
    assert!(response.score > -1.0e-6, "score was {}", response.score);
    assert!(!response.diagnostics.is_empty());
}

#[test]
fn rejects_target_absent_from_schema() {
    let error = LinearOperator::default()
        .discover(request("missing"))
        .unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn rejects_non_positive_minimum_variance() {
    let operator = LinearOperator {
        minimum_variance: 0.0,
    };
    let error = operator.discover(request("y")).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}
