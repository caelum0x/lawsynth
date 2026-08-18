use lawsynth_core::{
    DiagnosticSeverity, Diagnostics, ProgressError, ProgressStage, ProgressTracker,
    ResourceLimitError, ResourceLimits,
};

#[test]
fn resource_limits_reject_excess_work_before_execution() {
    let limits =
        ResourceLimits { max_samples: 4, max_columns: 2, max_features: 3, max_candidates: 1 };
    assert!(limits.validate_dataset(4, 2).is_ok());
    assert_eq!(
        limits.validate_feature_count(4),
        Err(ResourceLimitError::Exceeded { resource: "features", actual: 4, limit: 3 })
    );
}

#[test]
fn progress_is_monotonic_within_each_stage_but_not_globally() {
    let mut progress = ProgressTracker::default();
    let first = progress.report(ProgressStage::Profiling, 0.5, "scanning columns").unwrap();
    let second = progress.report(ProgressStage::Features, 0.1, "building terms").unwrap();
    assert_eq!((first.sequence, second.sequence), (0, 1));
    assert_eq!(
        progress.report(ProgressStage::Profiling, 0.4, "invalid"),
        Err(ProgressError::NonMonotonic)
    );
}

#[test]
fn diagnostics_are_structured_and_retain_failures() {
    let mut diagnostics = Diagnostics::default();
    diagnostics.warning("data.irregular_time", "using an irregular-grid derivative");
    diagnostics.error("resource.features", "feature limit exceeded");
    assert!(diagnostics.has_errors());
    assert_eq!(diagnostics.entries()[0].severity, DiagnosticSeverity::Warning);
    assert_eq!(diagnostics.entries()[1].code, "resource.features");
}
