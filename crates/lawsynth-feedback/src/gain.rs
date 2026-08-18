//! The result of a feedback design: the gain, the achieved spectrum, and — for
//! LQR — the solved Riccati matrix.

use lawsynth_koopman::{Complex, Matrix};

/// A designed state-feedback gain and the closed-loop spectrum it achieves.
///
/// The control law is `u = −K x`, so the closed-loop dynamics are `A − B K`.
/// [`achieved_poles`](Gain::achieved_poles) are the eigenvalues of `A − B K`
/// computed with the shared deterministic eigensolver, so a caller can confirm
/// the design is stable (or matches its targets) without re-deriving anything.
#[derive(Clone, Debug)]
pub struct Gain {
    /// The feedback gain `K`, shaped `m × n` (`m` inputs, `n` states).
    pub k: Matrix,
    /// Eigenvalues of the closed loop `A − B K`, in the eigensolver's canonical
    /// order (descending modulus, deterministic tie-breaks).
    pub achieved_poles: Vec<Complex>,
    /// The solved algebraic-Riccati matrix `P` (LQR only); `None` for pole
    /// placement, which does not form a value function.
    pub p: Option<Matrix>,
}

impl Gain {
    /// True when every achieved pole has strictly negative real part, up to a
    /// small absolute margin (i.e. the closed loop is Hurwitz).
    pub fn is_stable(&self, margin: f64) -> bool {
        self.achieved_poles.iter().all(|pole| pole.re < -margin)
    }
}
