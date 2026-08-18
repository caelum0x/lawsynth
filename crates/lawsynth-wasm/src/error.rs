use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WasmError {
    InvalidWorld(String),
    InvalidExpression(String),
    InvalidBundle(String),
    InvalidTrajectory(String),
    Simulation(String),
    MemoryLimit { requested: usize, available: usize },
    Unsupported(String),
}
impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidWorld(s) => write!(f, "invalid world: {s}"),
            Self::InvalidExpression(s) => write!(f, "invalid expression: {s}"),
            Self::InvalidBundle(s) => write!(f, "invalid bundle: {s}"),
            Self::InvalidTrajectory(s) => write!(f, "invalid trajectory: {s}"),
            Self::Simulation(s) => write!(f, "simulation error: {s}"),
            Self::MemoryLimit { requested, available } => {
                write!(f, "requested {requested} bytes but only {available} remain")
            }
            Self::Unsupported(s) => write!(f, "unsupported WASM surface: {s}"),
        }
    }
}
impl std::error::Error for WasmError {}
