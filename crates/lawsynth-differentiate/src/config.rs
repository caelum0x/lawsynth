use crate::DerivativeMethod;

#[derive(Clone, Debug, PartialEq)]
pub struct DerivativeConfig {
    pub method: DerivativeMethod,
}

impl Default for DerivativeConfig {
    fn default() -> Self {
        Self {
            method: DerivativeMethod::FiniteDifference,
        }
    }
}
