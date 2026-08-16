use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_features::FeatureLibrary;

#[test]
fn trigonometric_columns_satisfy_the_pythagorean_identity_per_sample() {
    let angle = Identifier::new("angle").unwrap();
    let data = Dataset::new(
        TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
        [NumericColumn::new(
            angle.clone(),
            vec![0.0, std::f64::consts::FRAC_PI_4, std::f64::consts::PI],
        )],
    )
    .unwrap();
    let matrix = FeatureLibrary::trigonometric([angle])
        .unwrap()
        .evaluate(&data)
        .unwrap();
    for row in matrix.rows {
        assert!((row[0].powi(2) + row[1].powi(2) - 1.0).abs() < 1e-12);
    }
}
