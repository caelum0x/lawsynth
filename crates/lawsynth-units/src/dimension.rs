/// Exponents in SI base-dimension order: length, mass, time, current,
/// temperature, amount, luminous intensity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Dimension([i8; 7]);

impl Dimension {
    pub const DIMENSIONLESS: Self = Self([0; 7]);
    pub const LENGTH: Self = Self([1, 0, 0, 0, 0, 0, 0]);
    pub const MASS: Self = Self([0, 1, 0, 0, 0, 0, 0]);
    pub const TIME: Self = Self([0, 0, 1, 0, 0, 0, 0]);

    pub fn exponents(self) -> [i8; 7] {
        self.0
    }

    pub fn multiply(self, other: Self) -> Option<Self> {
        self.combine(other, i8::checked_add)
    }

    pub fn divide(self, other: Self) -> Option<Self> {
        self.combine(other, i8::checked_sub)
    }

    pub fn pow(self, exponent: i8) -> Option<Self> {
        let mut result = [0; 7];
        for (target, source) in result.iter_mut().zip(self.0) {
            *target = source.checked_mul(exponent)?;
        }
        Some(Self(result))
    }

    fn combine(self, other: Self, operation: fn(i8, i8) -> Option<i8>) -> Option<Self> {
        let mut result = [0; 7];
        for ((target, left), right) in result.iter_mut().zip(self.0).zip(other.0) {
            *target = operation(left, right)?;
        }
        Some(Self(result))
    }
}
