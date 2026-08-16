/// Size and stride for aligned sliding windows over a Dataset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowConfig {
    pub width: usize,
    pub step: usize,
}

impl WindowConfig {
    pub fn new(width: usize, step: usize) -> Self {
        Self { width, step }
    }
}
