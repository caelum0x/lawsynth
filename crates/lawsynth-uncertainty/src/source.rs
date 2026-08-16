/// The origin of a quantified uncertainty contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SourceKind {
    Measurement,
    Parameter,
    Structural,
    Numerical,
    Sampling,
}

/// A named non-negative standard-deviation contribution.
#[derive(Clone, Debug, PartialEq)]
pub struct UncertaintySource {
    pub name: String,
    pub kind: SourceKind,
    pub standard_deviation: f64,
}

impl UncertaintySource {
    pub fn variance(&self) -> f64 {
        self.standard_deviation * self.standard_deviation
    }
}
