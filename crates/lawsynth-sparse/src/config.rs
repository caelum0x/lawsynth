#[derive(Clone, Debug, PartialEq)]
pub struct SparseConfig {
    pub threshold: f64,
    pub max_iterations: usize,
    pub ridge: f64,
}

impl Default for SparseConfig {
    fn default() -> Self {
        Self { threshold: 0.05, max_iterations: 20, ridge: 1e-10 }
    }
}
