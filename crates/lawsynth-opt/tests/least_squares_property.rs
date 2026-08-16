use lawsynth_opt::fit_affine;

#[test]
fn affine_fit_recovers_scale_and_offset_for_multiple_exact_linear_laws() {
    for (scale, offset) in [(2.0, -3.0), (-0.5, 7.0), (1.25, 0.0)] {
        let prediction = [-2.0, 0.0, 1.0, 4.0];
        let target = prediction.map(|value| scale * value + offset);
        let fit = fit_affine(&prediction, &target).unwrap();
        assert!((fit.scale - scale).abs() < 1e-12);
        assert!((fit.offset - offset).abs() < 1e-12);
        assert!(fit.mean_squared_error < 1e-24);
    }
}
