//! Linear-stability classification from Jacobian eigenvalues.
//!
//! The verdict is a pure function of the eigenvalues and a tolerance band around
//! the imaginary axis. An eigenvalue whose real part lies within `±band` is
//! treated as marginal (its sign is not resolvable), and any eigenvalue with a
//! marginal real part makes the linearization inconclusive — we say so rather
//! than committing to a definitive class.

use lawsynth_koopman::Complex;

/// The local behaviour of a fixed point, read off its Jacobian eigenvalues.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Classification {
    /// All eigenvalues real with `Re < 0`: trajectories decay without rotation.
    StableNode,
    /// All eigenvalues have `Re < 0` with a non-zero imaginary part: decaying
    /// oscillation (a stable focus/spiral).
    StableSpiral,
    /// All eigenvalues real with `Re > 0`: trajectories grow without rotation.
    UnstableNode,
    /// All eigenvalues have `Re > 0` with a non-zero imaginary part: growing
    /// oscillation (an unstable focus/spiral).
    UnstableSpiral,
    /// Real parts of mixed sign (all bounded away from zero): a saddle.
    Saddle,
    /// All eigenvalues on the imaginary axis (within the band) with non-zero
    /// imaginary part. The linearization suggests a center, but this is
    /// non-hyperbolic: the true nonlinear behaviour cannot be decided from the
    /// linearization alone.
    Center,
    /// At least one eigenvalue's real part sits inside the band (and the point is
    /// not a clean center). The fixed point is non-hyperbolic and linear
    /// stability analysis is inconclusive here.
    Marginal,
}

impl Classification {
    /// Whether linearization leaves the verdict genuinely undecided.
    pub fn is_inconclusive(self) -> bool {
        matches!(self, Self::Center | Self::Marginal)
    }
}

/// The sign of an eigenvalue's real part relative to the marginal band.
enum RealSign {
    Negative,
    Positive,
    Marginal,
}

fn real_sign(re: f64, band: f64) -> RealSign {
    if re > band {
        RealSign::Positive
    } else if re < -band {
        RealSign::Negative
    } else {
        RealSign::Marginal
    }
}

/// Classifies a fixed point from its Jacobian eigenvalues.
///
/// `band` is the half-width of the neighbourhood of the imaginary axis in which
/// a real part is treated as zero, and (symmetrically) the threshold above which
/// an imaginary part counts as genuine oscillation.
pub fn classify(eigenvalues: &[Complex], band: f64) -> Classification {
    let mut positive = 0usize;
    let mut negative = 0usize;
    let mut marginal = 0usize;
    let mut has_oscillation = false;

    for eigenvalue in eigenvalues {
        match real_sign(eigenvalue.re, band) {
            RealSign::Positive => positive += 1,
            RealSign::Negative => negative += 1,
            RealSign::Marginal => marginal += 1,
        }
        if eigenvalue.im.abs() > band {
            has_oscillation = true;
        }
    }

    if marginal > 0 {
        // Non-hyperbolic. Pure imaginary spectrum with oscillation reads as a
        // center; anything else is simply marginal / indeterminate.
        if positive == 0 && negative == 0 && has_oscillation {
            return Classification::Center;
        }
        return Classification::Marginal;
    }

    if positive > 0 && negative > 0 {
        Classification::Saddle
    } else if negative > 0 {
        if has_oscillation { Classification::StableSpiral } else { Classification::StableNode }
    } else if has_oscillation {
        Classification::UnstableSpiral
    } else {
        Classification::UnstableNode
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BAND: f64 = 1e-6;

    #[test]
    fn all_negative_real_is_a_stable_node() {
        let eigs = [Complex::real(-1.0), Complex::real(-2.0)];
        assert_eq!(classify(&eigs, BAND), Classification::StableNode);
    }

    #[test]
    fn negative_real_with_oscillation_is_a_stable_spiral() {
        let eigs = [Complex::new(-0.15, 0.98), Complex::new(-0.15, -0.98)];
        assert_eq!(classify(&eigs, BAND), Classification::StableSpiral);
    }

    #[test]
    fn all_positive_real_is_an_unstable_node() {
        let eigs = [Complex::real(1.0), Complex::real(2.0)];
        assert_eq!(classify(&eigs, BAND), Classification::UnstableNode);
    }

    #[test]
    fn positive_real_with_oscillation_is_an_unstable_spiral() {
        let eigs = [Complex::new(0.2, 1.0), Complex::new(0.2, -1.0)];
        assert_eq!(classify(&eigs, BAND), Classification::UnstableSpiral);
    }

    #[test]
    fn mixed_sign_real_is_a_saddle() {
        let eigs = [Complex::real(1.0), Complex::real(-1.0)];
        assert_eq!(classify(&eigs, BAND), Classification::Saddle);
    }

    #[test]
    fn pure_imaginary_is_a_center() {
        let eigs = [Complex::new(0.0, 1.0), Complex::new(0.0, -1.0)];
        assert_eq!(classify(&eigs, BAND), Classification::Center);
    }

    #[test]
    fn a_lone_zero_eigenvalue_is_marginal() {
        let eigs = [Complex::real(0.0)];
        assert_eq!(classify(&eigs, BAND), Classification::Marginal);
    }

    #[test]
    fn a_marginal_direction_mixed_with_a_stable_one_is_marginal() {
        let eigs = [Complex::real(-1.0), Complex::real(0.0)];
        assert_eq!(classify(&eigs, BAND), Classification::Marginal);
        assert!(classify(&eigs, BAND).is_inconclusive());
    }
}
