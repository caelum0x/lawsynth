use lawsynth_opt::{LbfgsConfig, ParameterBounds, lbfgs_minimize};

#[test]
fn lbfgs_converges_deterministically_with_analytic_gradient_and_bounds() {
    let minimize = || {
        lbfgs_minimize(
            &[4.0, -4.0],
            ParameterBounds::new(-5.0, 5.0).unwrap(),
            LbfgsConfig::default(),
            |point| {
                let dx = point[0] - 1.5;
                let dy = point[1] + 2.0;
                (dx * dx + dy * dy, vec![2.0 * dx, 2.0 * dy])
            },
        )
        .unwrap()
    };
    let result = minimize();
    assert_eq!(result, minimize());
    assert!((result[0] - 1.5).abs() < 1e-8);
    assert!((result[1] + 2.0).abs() < 1e-8);
}
