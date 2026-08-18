use lawsynth_data::Dataset;
use lawsynth_features::{FeatureConfig, FeatureLibrary, FeatureMatrix};

use crate::{ControlError, ControlSpec};

/// Builds the augmented candidate library `Θ(x, u)` for a controlled system.
///
/// The library is a polynomial expansion over the **combined** variable set
/// `[states.., controls..]` (see [`ControlSpec::variables`]). Because states and
/// controls share one polynomial basis, control-only terms (`u`, `u²`) and
/// state×control cross terms (`x·u`, `y·u`) appear alongside the ordinary state
/// terms. This is exactly the SINDYc augmentation, and it reuses the
/// `lawsynth-features` polynomial machinery rather than re-implementing it.
///
/// Term order is inherited from `lawsynth-features`, which is deterministic for
/// a fixed variable order — and the variable order is itself fixed by the spec.
pub(crate) fn build_augmented_library(
    spec: &ControlSpec,
    config: &FeatureConfig,
) -> Result<FeatureLibrary, ControlError> {
    let library = FeatureLibrary::polynomial(
        spec.variables(),
        config.polynomial_degree,
        config.include_constant,
    )?;
    Ok(library)
}

/// Evaluates the augmented library against every row of `dataset`.
///
/// The returned matrix rows are the design-matrix rows shared by all state
/// regressions. Controls contribute their *measured* values here — they are
/// used as library inputs only, and are never differentiated.
pub(crate) fn evaluate_library(
    library: &FeatureLibrary,
    dataset: &Dataset,
) -> Result<FeatureMatrix, ControlError> {
    Ok(library.evaluate(dataset)?)
}

#[cfg(test)]
mod tests {
    use lawsynth_core::Identifier;

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn augmented_library_contains_control_and_cross_terms() {
        let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
        let library = build_augmented_library(&spec, &FeatureConfig::default()).unwrap();
        let names = library.terms().iter().map(|term| term.name.clone()).collect::<Vec<_>>();
        // Control-only term, control squared, and state×control cross terms must
        // all be present in a degree-2 augmented expansion.
        assert!(names.iter().any(|name| name == "u"), "missing control term: {names:?}");
        assert!(names.iter().any(|name| name.contains('u') && name.contains('x')));
        assert!(names.iter().any(|name| name.contains('u') && name.contains('y')));
        // A control-squared term exists (its printed form multiplies u by u).
        assert!(
            names.iter().any(|name| name.matches('u').count() == 2),
            "missing u^2 term: {names:?}"
        );
    }

    #[test]
    fn augmented_library_is_deterministic() {
        let spec = ControlSpec::new([id("x"), id("y")], [id("u")]).unwrap();
        let first = build_augmented_library(&spec, &FeatureConfig::default()).unwrap();
        let second = build_augmented_library(&spec, &FeatureConfig::default()).unwrap();
        assert_eq!(first, second);
    }
}
