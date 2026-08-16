/// Boundary policy used by Python-facing constructors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PythonConfig {
    pub reject_unknown_keyword_data: bool,
}
impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            reject_unknown_keyword_data: true,
        }
    }
}
