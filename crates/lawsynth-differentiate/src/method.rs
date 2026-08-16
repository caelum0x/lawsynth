#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DerivativeMethod {
    #[default]
    FiniteDifference,
    SavitzkyGolay {
        window: usize,
    },
    NaturalCubicSpline,
    /// Periodic Fourier spectral derivative on a regular sample grid.
    Spectral,
    /// Total-variation denoising followed by a finite-difference derivative.
    ///
    /// `lambda` controls the amount of denoising and `iterations` bounds the
    /// deterministic ADMM solve. This is useful for piecewise-smooth signals
    /// whose direct finite differences are dominated by measurement noise.
    TotalVariation {
        lambda: f64,
        iterations: usize,
    },
}
