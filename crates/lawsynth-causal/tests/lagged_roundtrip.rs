use lawsynth_causal::{CausalConfig, granger_test, lagged_observations};
#[test]
fn lagged_histories_preserve_recency_and_granger_detects_predictive_input() {
    let x: Vec<f64> = (0..64).map(|i| (i as f64 / 3.0).sin()).collect();
    let mut y = vec![0.0; 64];
    for i in 1..64 {
        y[i] = 0.35 * y[i - 1] + 1.7 * x[i - 1];
    }
    let rows = lagged_observations(&y, 2).unwrap();
    assert_eq!(rows[0].history, vec![y[1], y[0]]);
    let r =
        granger_test(&x, &y, CausalConfig { max_lag: 1, min_samples: 20, ..Default::default() })
            .unwrap();
    assert!(r.unrestricted_sse < r.restricted_sse);
    assert!(r.f_statistic > 1.0);
}
