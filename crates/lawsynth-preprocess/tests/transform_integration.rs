use lawsynth_preprocess::{ImputationMethod, PreprocessError, impute_series};

#[test]
fn imputation_is_explicit_about_method_and_boundary_evidence() {
    let time = [0.0, 1.0, 3.0, 4.0];
    let values = [Some(0.0), None, Some(6.0), Some(8.0)];
    let (linear, report) = impute_series(&time, &values, ImputationMethod::Linear).unwrap();
    assert_eq!(linear, vec![0.0, 2.0, 6.0, 8.0]);
    assert_eq!(report.imputed_indices, vec![1]);

    let (mean, _) = impute_series(&time, &values, ImputationMethod::Mean).unwrap();
    assert_eq!(mean, vec![0.0, 14.0 / 3.0, 6.0, 8.0]);
    assert_eq!(
        impute_series(&time, &[None, Some(1.0), Some(2.0), Some(3.0)], ImputationMethod::Linear),
        Err(PreprocessError::MissingBoundaryValue)
    );
}
