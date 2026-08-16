#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewriteConfig {
    pub max_passes: usize,
}

impl Default for RewriteConfig {
    fn default() -> Self {
        Self { max_passes: 16 }
    }
}
