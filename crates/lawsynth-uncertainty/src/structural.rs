use crate::{SourceKind, UncertaintyError, UncertaintySource};

/// Explainable aggregation of explicitly declared structural-model alternatives.
#[derive(Clone, Debug, PartialEq)]
pub struct StructuralUncertainty {
    pub sources: Vec<UncertaintySource>,
}

impl StructuralUncertainty {
    pub fn new(sources: Vec<UncertaintySource>) -> Result<Self, UncertaintyError> {
        if sources.is_empty() {
            return Err(UncertaintyError::EmptyInput);
        }
        if sources.iter().any(|source| {
            source.kind != SourceKind::Structural
                || !source.standard_deviation.is_finite()
                || source.standard_deviation < 0.0
        }) {
            return Err(UncertaintyError::NonFiniteValue);
        }
        Ok(Self { sources })
    }
    pub fn combined_standard_deviation(&self) -> f64 {
        self.sources
            .iter()
            .map(UncertaintySource::variance)
            .sum::<f64>()
            .sqrt()
    }
}

/// Akaike-style normalized structural ambiguity from competing candidate scores.
pub fn structural_score(scores: &[f64]) -> Result<f64, UncertaintyError> {
    if scores.len() < 2 {
        return Err(UncertaintyError::TooFewSamples {
            minimum: 2,
            actual: scores.len(),
        });
    }
    if scores.iter().any(|score| !score.is_finite()) {
        return Err(UncertaintyError::NonFiniteValue);
    }
    let minimum = scores.iter().copied().fold(f64::INFINITY, f64::min);
    let weights: Vec<f64> = scores
        .iter()
        .map(|score| (-0.5 * (score - minimum)).exp())
        .collect();
    let total = weights.iter().sum::<f64>();
    Ok(1.0
        - weights
            .iter()
            .map(|weight| (weight / total).powi(2))
            .sum::<f64>())
}
