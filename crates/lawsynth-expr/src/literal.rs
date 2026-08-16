/// A finite scalar literal permitted in the expression IR.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Literal(f64);

impl Literal {
    pub fn new(value: f64) -> Option<Self> {
        value.is_finite().then_some(Self(value))
    }
    pub const fn value(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Literal {
    type Error = &'static str;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value).ok_or("literal must be finite")
    }
}
