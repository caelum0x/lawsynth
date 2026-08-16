/// Deterministic settings for bounded coordinate search.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordinateConfig {
    pub initial_step: f64,
    pub minimum_step: f64,
    pub max_iterations: usize,
}

impl Default for CoordinateConfig {
    fn default() -> Self {
        Self {
            initial_step: 1.0,
            minimum_step: 1e-8,
            max_iterations: 1_000,
        }
    }
}
