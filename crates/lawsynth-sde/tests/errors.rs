//! Error paths: `discover_sde` must fail loudly and specifically on malformed
//! inputs and configurations rather than returning a misleading model.

mod common;

use common::x;
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_sde::{BinRule, SdeConfig, SdeError, discover_sde};

fn dataset(times: Vec<f64>, values: Vec<f64>) -> Dataset {
    Dataset::new(
        TimeAxis::new(times).unwrap(),
        [NumericColumn::new(Identifier::new("x").unwrap(), values)],
    )
    .unwrap()
}

#[test]
fn single_point_path_is_rejected() {
    let data = dataset(vec![0.0], vec![1.0]);
    let error = discover_sde(&data, &SdeConfig::new()).unwrap_err();
    assert!(matches!(error, SdeError::TooFewSamples { rows: 1 }));
}

#[test]
fn irregular_time_axis_is_rejected_by_default() {
    let data = dataset(vec![0.0, 1.0, 3.0, 6.0, 10.0], vec![0.0, 0.1, 0.2, 0.1, 0.0]);
    let error = discover_sde(&data, &SdeConfig::new().with_min_bin_count(1)).unwrap_err();
    assert!(matches!(error, SdeError::IrregularTimeAxis));
}

#[test]
fn irregular_time_axis_is_allowed_when_not_required() {
    let data = dataset(vec![0.0, 1.0, 3.0, 6.0, 10.0], vec![0.0, 5.0, 0.2, 8.0, 0.0]);
    let mut config = SdeConfig::new()
        .with_bins(BinRule::Count(2))
        .with_min_bin_count(1)
        .with_polynomial_degree(1);
    config.require_regular_time = false;
    // Per-step Δt handles the irregular grid; the call succeeds.
    assert!(discover_sde(&data, &config).is_ok());
}

#[test]
fn degenerate_constant_state_is_rejected() {
    let data = dataset(vec![0.0, 1.0, 2.0, 3.0], vec![4.0, 4.0, 4.0, 4.0]);
    let error = discover_sde(&data, &SdeConfig::new().with_min_bin_count(1)).unwrap_err();
    assert!(matches!(error, SdeError::DegenerateState { .. }));
}

#[test]
fn too_few_populated_bins_is_rejected() {
    // A short path cannot fill enough bins to determine a degree-3 (4-term) fit
    // when the occupancy floor is set high.
    let data = dataset((0..6).map(|i| i as f64).collect(), vec![0.0, 1.0, 0.5, 2.0, 1.5, 3.0]);
    let config = SdeConfig::new().with_bins(BinRule::Count(24)).with_min_bin_count(1000);
    let error = discover_sde(&data, &config).unwrap_err();
    assert!(matches!(error, SdeError::TooFewPopulatedBins { .. }));
}

#[test]
fn unknown_state_column_is_rejected() {
    let data = dataset(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]);
    let config = SdeConfig::new().with_state_columns([Identifier::new("missing").unwrap()]);
    let error = discover_sde(&data, &config).unwrap_err();
    assert!(
        matches!(error, SdeError::UnknownStateColumn(id) if id == Identifier::new("missing").unwrap())
    );
}

#[test]
fn invalid_configuration_is_rejected() {
    let data = dataset(vec![0.0, 1.0, 2.0], vec![0.0, 1.0, 2.0]);

    let zero_degree = SdeConfig { polynomial_degree: 0, ..SdeConfig::new() };
    assert!(matches!(discover_sde(&data, &zero_degree), Err(SdeError::InvalidConfig(_))));

    let zero_bins = SdeConfig::new().with_bins(BinRule::Count(0));
    assert!(matches!(discover_sde(&data, &zero_bins), Err(SdeError::InvalidConfig(_))));

    let bad_width = SdeConfig::new().with_bins(BinRule::Width(-1.0));
    assert!(matches!(discover_sde(&data, &bad_width), Err(SdeError::InvalidConfig(_))));

    let zero_min = SdeConfig::new().with_min_bin_count(0);
    assert!(matches!(discover_sde(&data, &zero_min), Err(SdeError::InvalidConfig(_))));
}

#[test]
fn trajectory_count_must_divide_rows() {
    let data =
        dataset((0..7).map(|i| i as f64).collect(), (0..7).map(|i| i as f64 * 0.1).collect());
    // 7 rows cannot be split into 2 equal trajectories.
    let config = SdeConfig::new().with_min_bin_count(1).with_trajectories(2);
    assert!(matches!(discover_sde(&data, &config), Err(SdeError::InvalidConfig(_))));
}

#[test]
fn error_messages_are_descriptive() {
    let data = dataset(vec![0.0], vec![1.0]);
    let message = discover_sde(&data, &SdeConfig::new()).unwrap_err().to_string();
    assert!(message.contains("increment"), "message was: {message}");
    // The `x` helper is exercised here so the shared import is used.
    assert_eq!(x().as_str(), "x");
}
