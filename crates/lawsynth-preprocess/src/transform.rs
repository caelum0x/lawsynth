use crate::{DetrendReport, PreprocessReport, ResampleReport, ScaleReport};

/// One deterministic preprocessing operation in a reproducible pipeline.
#[derive(Clone, Debug, PartialEq)]
pub enum PreprocessStep {
    MovingAverage { radius: usize },
    DetrendLinear,
    ResampleLinear { target_time: Vec<f64> },
    Standardize,
}

/// Provenance emitted after a preprocessing operation completes.
#[derive(Clone, Debug, PartialEq)]
pub enum AppliedTransform {
    MovingAverage(PreprocessReport),
    DetrendLinear(DetrendReport),
    ResampleLinear(ResampleReport),
    Standardize(ScaleReport),
}
