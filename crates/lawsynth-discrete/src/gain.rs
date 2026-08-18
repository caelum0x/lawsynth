//! The result of a discrete LQR design: the gain, the achieved closed-loop
//! spectrum, and the solved DARE matrix.

use lawsynth_koopman::{Complex, Matrix};

/// A discrete state-feedback gain and the closed-loop spectrum it achieves.
///
/// The control law is `u = −K x`, so the closed-loop dynamics are `x_{k+1} =
/// (A − BK) x_k`. [`achieved_poles`](DiscreteGain::achieved_poles) are the
/// eigenvalues of `A − BK` computed with the shared deterministic eigensolver,
/// so a caller can confirm **discrete** stability — all eigenvalues strictly
/// inside the unit circle — without re-deriving anything.
#[derive(Clone, Debug)]
pub struct DiscreteGain {
    /// The feedback gain `K`, shaped `m × n` (`m` inputs, `n` states).
    pub k: Matrix,
    /// Eigenvalues of the closed loop `A − BK`, in the eigensolver's canonical
    /// order (descending modulus, deterministic tie-breaks).
    pub achieved_poles: Vec<Complex>,
    /// The solved discrete-algebraic-Riccati matrix `P` (symmetric).
    pub p: Matrix,
}

impl DiscreteGain {
    /// The spectral radius `max |λ|` of the closed loop `A − BK`.
    pub fn spectral_radius(&self) -> f64 {
        self.achieved_poles.iter().map(|pole| pole.abs()).fold(0.0, f64::max)
    }

    /// True when every closed-loop eigenvalue lies strictly inside the unit
    /// circle by at least `margin` (i.e. `|λ| < 1 − margin`), the discrete-time
    /// stability condition.
    pub fn is_stable(&self, margin: f64) -> bool {
        self.achieved_poles.iter().all(|pole| pole.abs() < 1.0 - margin)
    }
}
