//! Run the custom operator against a tiny in-memory dataset.
//!
//! ```bash
//! cargo run --example basic
//! ```

use lawsynth_custom_operator::LinearOperator;
use lawsynth_plugin_api::{
    AlgorithmPlugin, AlgorithmRequest, Column, DataBatch, DataSchema, ScalarType,
};

fn main() {
    // A perfectly linear relationship: y = 2x + 1.
    let schema = DataSchema {
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
    };
    let columns = vec![
        DataBatch::Float64(vec![0.0, 1.0, 2.0, 3.0, 4.0]),
        DataBatch::Float64(vec![1.0, 3.0, 5.0, 7.0, 9.0]),
    ];
    let request = AlgorithmRequest {
        schema,
        columns,
        target: "y".into(),
    };

    let response = LinearOperator::default()
        .discover(request)
        .expect("discovery should succeed for a clean linear dataset");

    println!("equation:    {}", response.equation);
    println!("score (−MSE): {}", response.score);
    for diagnostic in &response.diagnostics {
        println!("diagnostic:  {diagnostic}");
    }
}
