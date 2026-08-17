//! A minimal, dependency-free complex number for eigenvalues and modes.

use std::fmt;

/// A double-precision complex number `re + im·i`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

// The arithmetic is exposed as inherent methods (rather than `std::ops`) so the
// numeric kernels read as explicit method chains; the shadowing lint is
// intentionally waived for this value type.
#[allow(clippy::should_implement_trait)]
impl Complex {
    /// The complex number `re + im·i`.
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// A purely real complex number.
    pub const fn real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    /// The additive identity `0 + 0i`.
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    /// The multiplicative identity `1 + 0i`.
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    /// The complex conjugate `re - im·i`.
    pub fn conj(self) -> Self {
        Self { re: self.re, im: -self.im }
    }

    /// The modulus `sqrt(re² + im²)`.
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// The squared modulus `re² + im²`.
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// The argument (phase angle) in radians.
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    pub fn add(self, other: Self) -> Self {
        Self { re: self.re + other.re, im: self.im + other.im }
    }

    pub fn sub(self, other: Self) -> Self {
        Self { re: self.re - other.re, im: self.im - other.im }
    }

    pub fn mul(self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    /// Scales both components by a real factor.
    pub fn scale(self, factor: f64) -> Self {
        Self { re: self.re * factor, im: self.im * factor }
    }

    /// Complex division `self / other`.
    pub fn div(self, other: Self) -> Self {
        let denom = other.norm_sqr();
        Self {
            re: (self.re * other.re + self.im * other.im) / denom,
            im: (self.im * other.re - self.re * other.im) / denom,
        }
    }

    /// The principal complex square root.
    pub fn sqrt(self) -> Self {
        if self.re == 0.0 && self.im == 0.0 {
            return Self::ZERO;
        }
        let modulus = self.abs();
        let re = ((modulus + self.re) / 2.0).max(0.0).sqrt();
        let mut im = ((modulus - self.re) / 2.0).max(0.0).sqrt();
        if self.im < 0.0 {
            im = -im;
        }
        Self { re, im }
    }

    /// The natural logarithm on the principal branch.
    pub fn ln(self) -> Self {
        Self { re: self.abs().ln(), im: self.arg() }
    }

    /// True when both components are exactly zero.
    pub fn is_zero(self) -> bool {
        self.re == 0.0 && self.im == 0.0
    }
}

impl fmt::Display for Complex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.im >= 0.0 {
            write!(formatter, "{}+{}i", self.re, self.im)
        } else {
            write!(formatter, "{}-{}i", self.re, -self.im)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: Complex, b: Complex) -> bool {
        (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12
    }

    #[test]
    fn multiplies_and_divides_consistently() {
        let a = Complex::new(2.0, 3.0);
        let b = Complex::new(-1.0, 4.0);
        let product = a.mul(b);
        assert!(close(product, Complex::new(-14.0, 5.0)));
        // (a·b)/b == a.
        assert!(close(product.div(b), a));
    }

    #[test]
    fn square_root_of_negative_one_is_i() {
        let root = Complex::real(-1.0).sqrt();
        assert!(close(root, Complex::new(0.0, 1.0)));
    }

    #[test]
    fn square_root_squared_recovers_input() {
        let value = Complex::new(3.0, -4.0);
        let root = value.sqrt();
        assert!(close(root.mul(root), value));
    }

    #[test]
    fn log_inverts_the_modulus_and_argument() {
        let value = Complex::new(0.0, 2.0);
        let log = value.ln();
        assert!((log.re - 2.0_f64.ln()).abs() < 1e-12);
        assert!((log.im - std::f64::consts::FRAC_PI_2).abs() < 1e-12);
    }
}
