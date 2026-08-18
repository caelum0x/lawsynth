use crate::{CoordinateConfig, OptimizationError, ParameterBounds, coordinate_minimize};

/// Result of enumerating finite discrete modes and optimizing continuous values for each.
#[derive(Clone, Debug, PartialEq)]
pub struct MixedResult<T> {
    pub discrete: T,
    pub continuous: Vec<f64>,
    pub objective: f64,
}

/// Finds the best finite discrete mode, calibrating continuous values by coordinate search.
pub fn mixed_minimize<T, F>(
    modes: impl IntoIterator<Item = T>,
    initial: &[f64],
    bounds: ParameterBounds,
    config: CoordinateConfig,
    objective: F,
) -> Result<MixedResult<T>, OptimizationError>
where
    T: Clone,
    F: Fn(&T, &[f64]) -> f64,
{
    let mut best: Option<MixedResult<T>> = None;
    for mode in modes {
        let result = coordinate_minimize(initial, bounds, config, |continuous| {
            objective(&mode, continuous)
        })?;
        if best.as_ref().is_none_or(|best| result.objective < best.objective) {
            best = Some(MixedResult {
                discrete: mode,
                continuous: result.parameters,
                objective: result.objective,
            });
        }
    }
    best.ok_or(OptimizationError::EmptyInput)
}
