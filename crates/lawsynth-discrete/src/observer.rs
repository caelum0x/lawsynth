//! The result of a discrete observer / Kalman design: the gain, the achieved
//! error spectrum, and — for the Kalman filter — the solved error covariance.

use lawsynth_koopman::{Complex, Matrix};

/// How an observer gain was designed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObserverMethod {
    /// Steady-state discrete Kalman filter (dual filter DARE).
    Kalman,
    /// Discrete Luenberger observer by dual pole placement in the z-plane.
    PolePlacement,
}

/// A discrete observer gain and the error spectrum it achieves.
///
/// The observer is a corrected model copy
/// `x̂_{k+1} = A x̂_k + B u_k + L (y_k − C x̂_k)`, so the estimation error
/// `e = x − x̂` obeys `e_{k+1} = (A − LC) e_k`.
/// [`error_poles`](DiscreteObserver::error_poles) are the eigenvalues of
/// `A − LC` from the shared deterministic eigensolver, so a caller can confirm
/// convergence — all eigenvalues strictly inside the unit circle.
#[derive(Clone, Debug)]
pub struct DiscreteObserver {
    /// The observer gain `L`, shaped `n × p` (`n` states, `p` outputs).
    pub l: Matrix,
    /// Eigenvalues of the error dynamics `A − LC`, in the eigensolver's
    /// canonical order.
    pub error_poles: Vec<Complex>,
    /// The solved error covariance `P` (Kalman only); `None` for pole placement,
    /// which forms no covariance.
    pub p: Option<Matrix>,
    /// The method that produced this gain.
    pub method: ObserverMethod,
}

impl DiscreteObserver {
    /// The spectral radius `max |λ|` of the error dynamics `A − LC`.
    pub fn spectral_radius(&self) -> f64 {
        self.error_poles.iter().map(|pole| pole.abs()).fold(0.0, f64::max)
    }

    /// True when every error eigenvalue lies strictly inside the unit circle by
    /// at least `margin` (i.e. `|λ| < 1 − margin`), so `x̂ → x`.
    pub fn is_convergent(&self, margin: f64) -> bool {
        self.error_poles.iter().all(|pole| pole.abs() < 1.0 - margin)
    }
}
