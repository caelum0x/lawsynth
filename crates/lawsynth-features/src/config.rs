#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FeatureConfig {
    pub polynomial_degree: usize,
    pub include_constant: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            polynomial_degree: 2,
            include_constant: true,
        }
    }
}
