use lawsynth_differentiate::savgol_series;

#[test]
fn exactly_recovers_the_slope_of_affine_signals_on_irregular_grids() {
    let grids = [
        vec![0.0, 0.25, 0.8, 1.5, 2.75],
        vec![-3.0, -0.5, 0.0, 0.1, 8.0],
    ];
    for time in grids {
        for (slope, intercept) in [(2.5, -1.0), (-0.75, 3.5)] {
            let values = time
                .iter()
                .map(|sample| slope * sample + intercept)
                .collect::<Vec<_>>();
            let derivative = savgol_series(&time, &values, 3).unwrap();
            assert!(
                derivative
                    .iter()
                    .all(|actual| (*actual - slope).abs() < 1e-11)
            );
        }
    }
}
