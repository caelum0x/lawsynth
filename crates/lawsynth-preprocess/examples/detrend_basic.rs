use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::detrend_linear;

fn main() {
    let data = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0]).unwrap(),
        [NumericColumn::new(
            Identifier::new("signal").unwrap(),
            vec![2.0, 4.1, 6.0, 8.2],
        )],
    )
    .unwrap();
    let (residual, report) = detrend_linear(&data).unwrap();
    println!(
        "removed slopes {:?}; residual data fingerprint {}",
        report.slope,
        residual.fingerprint()
    );
}
