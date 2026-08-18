//! Endpoint classification: which attractor (if any) a trajectory reaches.

use crate::config::BasinConfig;
use crate::integrate::Flow;
use crate::report::Label;

/// Integrates one initial condition forward and classifies its fate.
///
/// The trajectory is checked for convergence to a recognized attractor at every
/// step; the first time it comes within `convergence_tolerance` (`‖·‖∞`) of an
/// attractor it is labelled with that attractor's index (nearest wins, lowest
/// index on a tie). If it leaves the escape region or diverges it is `Escaped`;
/// if neither happens within `max_time` it is `Undetermined`. Classification is
/// never forced.
pub(crate) fn classify_trajectory(
    flow: &Flow<'_>,
    initial: &[f64],
    attractors: &[Vec<f64>],
    config: &BasinConfig,
) -> Label {
    let tolerance = config.convergence_tolerance();

    let mut state = initial.to_vec();
    if let Some(index) = nearest_attractor(&state, attractors, tolerance) {
        return Label::Attractor(index);
    }

    for _ in 0..config.step_count() {
        state = match flow.step(&state, config.dt()) {
            Ok(next) => next,
            // The field is undefined here (e.g. log of a non-positive argument):
            // the flow has left the valid region, which we treat as an escape.
            Err(_) => return Label::Escaped,
        };

        if state.iter().any(|value| !value.is_finite()) || escaped(&state, config) {
            return Label::Escaped;
        }
        if let Some(index) = nearest_attractor(&state, attractors, tolerance) {
            return Label::Attractor(index);
        }
    }

    Label::Undetermined
}

/// Returns the index of the nearest attractor within `tolerance` (`‖·‖∞`), or
/// `None` if none is that close. Ties resolve to the lowest index.
fn nearest_attractor(state: &[f64], attractors: &[Vec<f64>], tolerance: f64) -> Option<usize> {
    let mut best: Option<(usize, f64)> = None;
    for (index, attractor) in attractors.iter().enumerate() {
        let distance = chebyshev(state, attractor);
        if distance <= tolerance {
            match best {
                Some((_, best_distance)) if best_distance <= distance => {}
                _ => best = Some((index, distance)),
            }
        }
    }
    best.map(|(index, _)| index)
}

/// The `‖a − b‖∞` (Chebyshev) distance between two coordinate vectors.
fn chebyshev(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(&x, &y)| (x - y).abs()).fold(0.0, f64::max)
}

/// Whether `state` has left the escape region: the search box padded on every
/// axis by `escape_margin`, or any coordinate beyond `divergence_limit`.
fn escaped(state: &[f64], config: &BasinConfig) -> bool {
    let margin = config.escape_margin();
    let limit = config.divergence_limit();
    state.iter().zip(config.search_box()).any(|(&value, &(lower, upper))| {
        value < lower - margin || value > upper + margin || value.abs() > limit
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_attractor_picks_the_closest_within_tolerance() {
        let attractors = vec![vec![-1.0], vec![1.0]];
        assert_eq!(nearest_attractor(&[0.9995], &attractors, 1e-3), Some(1));
        assert_eq!(nearest_attractor(&[-1.0005], &attractors, 1e-3), Some(0));
        assert_eq!(nearest_attractor(&[0.0], &attractors, 1e-3), None);
    }

    #[test]
    fn nearest_attractor_breaks_ties_to_lowest_index() {
        // Two coincident attractors, both within tolerance: lowest index wins.
        let attractors = vec![vec![0.0], vec![0.0]];
        assert_eq!(nearest_attractor(&[0.0], &attractors, 1e-3), Some(0));
    }

    #[test]
    fn chebyshev_is_the_max_coordinate_gap() {
        assert_eq!(chebyshev(&[0.0, 0.0], &[0.3, -0.7]), 0.7);
    }
}
