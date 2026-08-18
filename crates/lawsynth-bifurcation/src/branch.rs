//! Deterministic assembly of fixed-point branches across the parameter grid.
//!
//! Consecutive parameter samples are stitched into branches by nearest-coordinate
//! matching: at each step every active branch is greedily paired with the closest
//! not-yet-claimed fixed point within a tolerance, smallest distance first and
//! deterministic ties. A branch with no match ends; an unmatched fixed point
//! starts a new branch. The whole procedure is a pure function of the samples.

use std::cmp::Ordering;

use crate::report::{Branch, BranchPoint, ParameterSample};

/// The parameter-sample index span a branch occupies, `(first, last)`.
pub type BranchSpan = (usize, usize);

/// Euclidean distance between two coordinate vectors (assumed equal length).
fn distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(&x, &y)| (x - y) * (x - y)).sum::<f64>().sqrt()
}

/// Total order on `f64` for deterministic sorting (no `NaN` ambiguity).
fn total(a: f64, b: f64) -> Ordering {
    a.total_cmp(&b)
}

fn branch_point(sample: &ParameterSample, fixed_point_index: usize) -> BranchPoint {
    let fixed_point = &sample.report.fixed_points[fixed_point_index];
    BranchPoint {
        parameter_value: sample.parameter_value,
        coordinates: fixed_point.coordinates.clone(),
        eigenvalues: fixed_point.eigenvalues.clone(),
        classification: fixed_point.classification,
    }
}

/// One branch still open for continuation at the current step.
struct Active {
    branch: usize,
    last: Vec<f64>,
}

/// Assembles branches from `samples`, returning the branches (in creation order)
/// and, for each branch, the `(first, last)` sample index it spans.
pub fn assemble_branches(
    samples: &[ParameterSample],
    match_tolerance: f64,
) -> (Vec<Branch>, Vec<BranchSpan>) {
    let mut branches: Vec<Branch> = Vec::new();
    let mut spans: Vec<BranchSpan> = Vec::new();
    let mut actives: Vec<Active> = Vec::new();

    for (sample_index, sample) in samples.iter().enumerate() {
        let fixed_points = &sample.report.fixed_points;

        if actives.is_empty() {
            actives = start_branches(sample, sample_index, &mut branches, &mut spans);
            continue;
        }

        // Rank every (active, fixed point) pair within the tolerance.
        let mut candidates: Vec<(f64, usize, usize, usize)> = Vec::new();
        for (active_index, active) in actives.iter().enumerate() {
            for (fixed_point_index, fixed_point) in fixed_points.iter().enumerate() {
                let gap = distance(&active.last, &fixed_point.coordinates);
                if gap <= match_tolerance {
                    candidates.push((gap, active.branch, active_index, fixed_point_index));
                }
            }
        }
        candidates.sort_by(|a, b| total(a.0, b.0).then(a.1.cmp(&b.1)).then(a.3.cmp(&b.3)));

        let mut active_used = vec![false; actives.len()];
        let mut fixed_point_used = vec![false; fixed_points.len()];
        let mut next_actives: Vec<Active> = Vec::new();

        for &(_, branch_id, active_index, fixed_point_index) in &candidates {
            if active_used[active_index] || fixed_point_used[fixed_point_index] {
                continue;
            }
            active_used[active_index] = true;
            fixed_point_used[fixed_point_index] = true;

            let point = branch_point(sample, fixed_point_index);
            let coordinates = point.coordinates.clone();
            branches[branch_id].points.push(point);
            spans[branch_id].1 = sample_index;
            next_actives.push(Active { branch: branch_id, last: coordinates });
        }

        // Any fixed point with no predecessor starts a fresh branch.
        for (fixed_point_index, used) in fixed_point_used.iter().enumerate() {
            if *used {
                continue;
            }
            let branch_id = branches.len();
            branches.push(Branch {
                id: branch_id,
                points: vec![branch_point(sample, fixed_point_index)],
            });
            spans.push((sample_index, sample_index));
            next_actives.push(Active {
                branch: branch_id,
                last: fixed_points[fixed_point_index].coordinates.clone(),
            });
        }

        // Unmatched active branches simply end (not carried into next_actives).
        actives = next_actives;
    }

    (branches, spans)
}

/// Opens a new branch for every fixed point in `sample`.
fn start_branches(
    sample: &ParameterSample,
    sample_index: usize,
    branches: &mut Vec<Branch>,
    spans: &mut Vec<BranchSpan>,
) -> Vec<Active> {
    let mut actives = Vec::new();
    for fixed_point_index in 0..sample.report.fixed_points.len() {
        let branch_id = branches.len();
        branches
            .push(Branch { id: branch_id, points: vec![branch_point(sample, fixed_point_index)] });
        spans.push((sample_index, sample_index));
        actives.push(Active {
            branch: branch_id,
            last: sample.report.fixed_points[fixed_point_index].coordinates.clone(),
        });
    }
    actives
}

#[cfg(test)]
mod tests {
    use super::*;
    use lawsynth_core::Identifier;
    use lawsynth_koopman::Complex;
    use lawsynth_stability::{Classification, FixedPoint, StabilityReport};

    fn sample(parameter_value: f64, coords: &[&[f64]]) -> ParameterSample {
        let fixed_points = coords
            .iter()
            .map(|coordinates| FixedPoint {
                coordinates: coordinates.to_vec(),
                eigenvalues: vec![Complex::real(-1.0)],
                classification: Classification::StableNode,
            })
            .collect();
        ParameterSample {
            parameter_value,
            report: StabilityReport {
                states: vec![Identifier::new("x").unwrap()],
                fixed_points,
                seeds_total: 1,
                seeds_converged: 1,
            },
        }
    }

    #[test]
    fn a_single_moving_root_forms_one_branch() {
        let samples =
            vec![sample(0.0, &[&[0.0]]), sample(1.0, &[&[0.05]]), sample(2.0, &[&[0.10]])];
        let (branches, spans) = assemble_branches(&samples, 0.5);
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].points.len(), 3);
        assert_eq!(spans[0], (0, 2));
    }

    #[test]
    fn a_root_that_appears_starts_a_new_branch() {
        let samples = vec![sample(0.0, &[&[0.0]]), sample(1.0, &[&[0.0], &[2.0]])];
        let (branches, spans) = assemble_branches(&samples, 0.5);
        assert_eq!(branches.len(), 2);
        assert_eq!(spans[1], (1, 1));
    }

    #[test]
    fn a_root_that_jumps_beyond_tolerance_breaks_the_branch() {
        let samples = vec![sample(0.0, &[&[0.0]]), sample(1.0, &[&[5.0]])];
        let (branches, _) = assemble_branches(&samples, 0.5);
        // The distant root cannot continue the first branch, so there are two.
        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].points.len(), 1);
        assert_eq!(branches[1].points.len(), 1);
    }
}
