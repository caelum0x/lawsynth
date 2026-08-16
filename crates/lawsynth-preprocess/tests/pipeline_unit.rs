use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::{AppliedTransform, PreprocessPipeline, PreprocessStep};

#[test]
fn pipeline_runs_all_steps_in_order_and_retains_provenance_chain() {
    let dataset = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
        [NumericColumn::new(
            Identifier::new("x").unwrap(),
            vec![1.0, 3.0, 5.0],
        )],
    )
    .unwrap();
    let pipeline = PreprocessPipeline::new([
        PreprocessStep::MovingAverage { radius: 1 },
        PreprocessStep::Standardize,
    ]);
    let (output, reports) = pipeline.apply(&dataset).unwrap();
    assert_eq!(reports.len(), 2);
    let AppliedTransform::MovingAverage(smooth) = &reports[0] else {
        panic!("expected smoothing report")
    };
    let AppliedTransform::Standardize(scale) = &reports[1] else {
        panic!("expected scale report")
    };
    assert_eq!(smooth.output_fingerprint, scale.input_fingerprint);
    assert_eq!(scale.output_fingerprint, output.fingerprint());
}
