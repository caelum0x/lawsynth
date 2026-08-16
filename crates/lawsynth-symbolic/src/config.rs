#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolicConfig {
    pub max_depth: usize,
    pub max_candidates: usize,
    pub include_products: bool,
}

impl Default for SymbolicConfig {
    fn default() -> Self {
        Self {
            max_depth: 2,
            max_candidates: 256,
            include_products: true,
        }
    }
}
