//! Eigenvalue-crossing and fold detection along assembled branches.
//!
//! Two complementary detectors run:
//!
//! - **Crossings on a persisting branch.** Where the dominant eigenvalue's real
//!   part changes sign between consecutive points, a stability change occurred.
//!   The critical parameter is localized by bisection on that sign, and the event
//!   is a Hopf if the crossing eigenvalue carries a non-zero imaginary part, else
//!   a real zero-eigenvalue fold.
//! - **Folds at a branch birth/death.** Where a branch appears or disappears with
//!   a near-zero real eigenvalue, fixed points have collided (a saddle-node-type
//!   fold). The critical parameter is localized by bisection on fixed-point
//!   existence.

use lawsynth_koopman::Complex;
use lawsynth_stability::{FixedPoint, StabilityReport};

use crate::branch::BranchSpan;
use crate::context::FieldContext;
use crate::error::BifurcationError;
use crate::report::{Bifurcation, BifurcationKind, Branch, BranchPoint, Localization};
use crate::sweep::Sweep;

/// The dominant eigenvalue: the one with the greatest real part, ties broken by
/// larger `|Im|` then larger `Im` (so the choice is deterministic and prefers the
/// `+im` member of a conjugate pair).
fn dominant(eigenvalues: &[Complex]) -> Option<Complex> {
    eigenvalues.iter().copied().max_by(|a, b| {
        a.re.total_cmp(&b.re)
            .then_with(|| a.im.abs().total_cmp(&b.im.abs()))
            .then_with(|| a.im.total_cmp(&b.im))
    })
}

/// The dominant eigenvalue's real part, or `0.0` when the spectrum is empty.
fn dominant_real(point: &BranchPoint) -> f64 {
    dominant(&point.eigenvalues).map(|value| value.re).unwrap_or(0.0)
}

/// The side of the imaginary axis a real part sits on relative to `band`:
/// `+1` right, `-1` left, `0` inside the band.
fn side(real_part: f64, band: f64) -> i8 {
    if real_part > band {
        1
    } else if real_part < -band {
        -1
    } else {
        0
    }
}

/// Euclidean distance between two equal-length coordinate vectors.
fn distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(&x, &y)| (x - y) * (x - y)).sum::<f64>().sqrt()
}

/// The fixed point closest to `reference` (first one wins ties).
fn nearest<'a>(report: &'a StabilityReport, reference: &[f64]) -> Option<&'a FixedPoint> {
    report.fixed_points.iter().min_by(|a, b| {
        distance(&a.coordinates, reference).total_cmp(&distance(&b.coordinates, reference))
    })
}

/// The fixed point whose dominant eigenvalue is closest to the imaginary axis,
/// together with that eigenvalue — the candidate marginal point at a fold.
fn most_marginal(report: &StabilityReport) -> Option<(&FixedPoint, Complex)> {
    report
        .fixed_points
        .iter()
        .filter_map(|fixed_point| {
            dominant(&fixed_point.eigenvalues).map(|value| (fixed_point, value))
        })
        .min_by(|a, b| a.1.re.abs().total_cmp(&b.1.re.abs()))
}

/// Classifies a crossing eigenvalue as Hopf (complex) or Fold (real).
fn kind_of(eigenvalue: Complex, sweep: &Sweep) -> BifurcationKind {
    if eigenvalue.im.abs() > sweep.imaginary_tolerance() {
        BifurcationKind::Hopf
    } else {
        BifurcationKind::Fold
    }
}

/// Detects sign changes of the dominant real part along each branch.
pub(crate) fn detect_crossings(
    context: &FieldContext<'_>,
    sweep: &Sweep,
    branches: &[Branch],
) -> Result<Vec<Bifurcation>, BifurcationError> {
    let band = sweep.crossing_band();
    let mut found = Vec::new();
    for branch in branches {
        let mut last_strict_side = 0i8;
        let mut last_strict_index = 0usize;
        for (index, point) in branch.points.iter().enumerate() {
            let current = side(dominant_real(point), band);
            if current == 0 {
                continue;
            }
            if last_strict_side != 0 && current != last_strict_side {
                found.push(localize_crossing(context, sweep, branch, last_strict_index, index)?);
            }
            last_strict_side = current;
            last_strict_index = index;
        }
    }
    Ok(found)
}

/// Bisects on the dominant real part between two bracketing branch points.
fn localize_crossing(
    context: &FieldContext<'_>,
    sweep: &Sweep,
    branch: &Branch,
    low_index: usize,
    high_index: usize,
) -> Result<Bifurcation, BifurcationError> {
    let band = sweep.crossing_band();
    let low_point = &branch.points[low_index];
    let side_low = side(dominant_real(low_point), band);

    let mut low = low_point.parameter_value;
    let mut high = branch.points[high_index].parameter_value;
    let mut reference = low_point.coordinates.clone();

    for _ in 0..sweep.localization_iterations() {
        let mid = 0.5 * (low + high);
        let report = context.at(mid)?;
        match nearest(&report, &reference) {
            Some(fixed_point) => {
                let real = dominant(&fixed_point.eigenvalues).map(|value| value.re).unwrap_or(0.0);
                if side(real, band) == side_low {
                    low = mid;
                    reference = fixed_point.coordinates.clone();
                } else {
                    high = mid;
                }
            }
            None => high = mid,
        }
    }

    let critical = 0.5 * (low + high);
    let report = context.at(critical)?;
    let (fixed_point, eigenvalue) = match nearest(&report, &reference) {
        Some(fixed_point) => (
            fixed_point.coordinates.clone(),
            dominant(&fixed_point.eigenvalues).unwrap_or(Complex::ZERO),
        ),
        None => (reference, Complex::ZERO),
    };

    Ok(Bifurcation {
        branch_id: branch.id,
        parameter_value: critical,
        kind: kind_of(eigenvalue, sweep),
        localization: Localization::BisectionOnRealPart,
        fixed_point,
        eigenvalue,
    })
}

/// Detects folds where a branch is born or dies with a near-zero real eigenvalue.
pub(crate) fn detect_folds(
    context: &FieldContext<'_>,
    sweep: &Sweep,
    samples_counts: &[usize],
    grid: &[f64],
    branches: &[Branch],
    spans: &[BranchSpan],
) -> Result<Vec<Bifurcation>, BifurcationError> {
    let mut found = Vec::new();
    for (branch, &(first, last)) in branches.iter().zip(spans) {
        if first > 0 && samples_counts[first] > samples_counts[first - 1] {
            if let Some(bifurcation) = localize_fold(
                context,
                sweep,
                branch,
                grid[first - 1],
                grid[first],
                samples_counts[first],
                Endpoint::Birth,
            )? {
                found.push(bifurcation);
            }
        }
        if last + 1 < grid.len() && samples_counts[last] > samples_counts[last + 1] {
            if let Some(bifurcation) = localize_fold(
                context,
                sweep,
                branch,
                grid[last],
                grid[last + 1],
                samples_counts[last],
                Endpoint::Death,
            )? {
                found.push(bifurcation);
            }
        }
    }
    Ok(found)
}

/// Which end of a branch a fold is being localized at.
#[derive(Clone, Copy)]
enum Endpoint {
    /// The branch appears (fixed points exist at the higher parameter value).
    Birth,
    /// The branch disappears (fixed points exist at the lower parameter value).
    Death,
}

/// Bisects on fixed-point existence to localize a fold, gating on a near-zero
/// real eigenvalue so an ordinary box-boundary crossing is not mislabelled.
fn localize_fold(
    context: &FieldContext<'_>,
    sweep: &Sweep,
    branch: &Branch,
    lower: f64,
    upper: f64,
    threshold: usize,
    endpoint: Endpoint,
) -> Result<Option<Bifurcation>, BifurcationError> {
    let mut low = lower;
    let mut high = upper;
    for _ in 0..sweep.localization_iterations() {
        let mid = 0.5 * (low + high);
        let present = context.at(mid)?.fixed_points.len() >= threshold;
        // "Present" means fixed points exist. For a birth they exist at the upper
        // end, for a death at the lower end; move the boundary accordingly.
        match endpoint {
            Endpoint::Birth => {
                if present {
                    high = mid;
                } else {
                    low = mid;
                }
            }
            Endpoint::Death => {
                if present {
                    low = mid;
                } else {
                    high = mid;
                }
            }
        }
    }

    let critical = 0.5 * (low + high);
    // Evaluate just inside the region where the fixed points exist.
    let present_side = match endpoint {
        Endpoint::Birth => high,
        Endpoint::Death => low,
    };
    let report = context.at(present_side)?;
    let Some((fixed_point, eigenvalue)) = most_marginal(&report) else {
        return Ok(None);
    };
    if eigenvalue.re.abs() > sweep.fold_eigenvalue_tolerance() {
        return Ok(None);
    }
    Ok(Some(Bifurcation {
        branch_id: branch.id,
        parameter_value: critical,
        kind: kind_of(eigenvalue, sweep),
        localization: Localization::BisectionOnExistence,
        fixed_point: fixed_point.coordinates.clone(),
        eigenvalue,
    }))
}

/// Merges near-duplicate bifurcations (same kind, close in parameter and
/// coordinates) and returns the survivors ordered by ascending parameter value.
pub(crate) fn deduplicate(mut candidates: Vec<Bifurcation>, sweep: &Sweep) -> Vec<Bifurcation> {
    candidates.sort_by(|a, b| {
        a.parameter_value
            .total_cmp(&b.parameter_value)
            .then((a.kind as u8).cmp(&(b.kind as u8)))
            .then(a.branch_id.cmp(&b.branch_id))
    });
    let mut kept: Vec<Bifurcation> = Vec::new();
    for candidate in candidates {
        let duplicate = kept.iter().any(|existing| {
            existing.kind == candidate.kind
                && (existing.parameter_value - candidate.parameter_value).abs()
                    <= sweep.dedup_parameter_tolerance()
                && distance(&existing.fixed_point, &candidate.fixed_point)
                    <= sweep.dedup_coordinate_tolerance()
        });
        if !duplicate {
            kept.push(candidate);
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominant_prefers_greatest_real_part() {
        let eigs = [Complex::new(-1.0, 0.0), Complex::new(0.5, 2.0)];
        assert_eq!(dominant(&eigs).unwrap().re, 0.5);
    }

    #[test]
    fn dominant_breaks_conjugate_ties_toward_positive_imaginary() {
        let eigs = [Complex::new(0.0, -1.0), Complex::new(0.0, 1.0)];
        assert_eq!(dominant(&eigs).unwrap().im, 1.0);
    }

    #[test]
    fn side_respects_the_band() {
        assert_eq!(side(1.0, 1e-9), 1);
        assert_eq!(side(-1.0, 1e-9), -1);
        assert_eq!(side(1e-12, 1e-9), 0);
    }

    #[test]
    fn kind_splits_hopf_from_fold_by_imaginary_part() {
        let sweep = Sweep::new(-1.0, 1.0, 4);
        assert_eq!(kind_of(Complex::new(0.0, 1.0), &sweep), BifurcationKind::Hopf);
        assert_eq!(kind_of(Complex::new(0.0, 0.0), &sweep), BifurcationKind::Fold);
    }
}
