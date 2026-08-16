/// Ordered execution stages exposed for checkpoints, progress, and diagnostics.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum DiscoveryStage {
    Validate,
    Preprocess,
    Profile,
    Differentiate,
    GenerateFeatures,
    FitLaws,
    Score,
    Finalize,
}

impl DiscoveryStage {
    pub const fn all() -> [Self; 8] {
        [
            Self::Validate,
            Self::Preprocess,
            Self::Profile,
            Self::Differentiate,
            Self::GenerateFeatures,
            Self::FitLaws,
            Self::Score,
            Self::Finalize,
        ]
    }
}
