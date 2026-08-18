//! Deterministic seed generation for the Newton search.
//!
//! Seeds are a fixed lattice over the search box plus the origin — never random
//! and never derived from wall-clock or hash iteration order. For `grid` samples
//! per axis and `n` states the lattice has `grid^n` points, enumerated in a
//! fixed row-major order (first axis outermost), so identical configs yield an
//! identical seed list.

/// Even samples across `[lower, upper]`.
///
/// A resolution of one collapses to the interval midpoint; a degenerate interval
/// (`lower == upper`) collapses to that single value.
fn axis_samples(lower: f64, upper: f64, resolution: usize) -> Vec<f64> {
    if resolution <= 1 || lower == upper {
        return vec![(lower + upper) / 2.0];
    }
    (0..resolution)
        .map(|index| {
            let fraction = index as f64 / (resolution - 1) as f64;
            lower + (upper - lower) * fraction
        })
        .collect()
}

/// Builds the deterministic seed set: the per-axis lattice over `search_box`
/// plus the origin (appended once if the lattice does not already contain it).
pub(crate) fn seed_points(search_box: &[(f64, f64)], resolution: usize) -> Vec<Vec<f64>> {
    let axes: Vec<Vec<f64>> =
        search_box.iter().map(|&(lower, upper)| axis_samples(lower, upper, resolution)).collect();

    // Cartesian product in fixed row-major order (first axis varies slowest).
    let mut points: Vec<Vec<f64>> = vec![Vec::new()];
    for axis in &axes {
        let mut next = Vec::with_capacity(points.len() * axis.len());
        for prefix in &points {
            for &value in axis {
                let mut extended = prefix.clone();
                extended.push(value);
                next.push(extended);
            }
        }
        points = next;
    }

    let origin = vec![0.0; search_box.len()];
    if !points.iter().any(|point| point == &origin) {
        points.push(origin);
    }
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_resolution_uses_midpoint() {
        assert_eq!(axis_samples(-2.0, 4.0, 1), vec![1.0]);
    }

    #[test]
    fn samples_span_the_interval_inclusively() {
        assert_eq!(axis_samples(0.0, 1.0, 3), vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn lattice_has_grid_pow_dim_points_plus_origin() {
        // 3 samples per axis, 2 axes; origin is not on this even lattice, so +1.
        let seeds = seed_points(&[(-1.0, 1.0), (-1.0, 1.0)], 3);
        // 3x3 lattice includes (0,0), so no extra origin is appended.
        assert_eq!(seeds.len(), 9);
        assert!(seeds.contains(&vec![0.0, 0.0]));
    }

    #[test]
    fn origin_is_appended_when_missing_from_lattice() {
        // An even lattice on [1, 2] never hits 0, so the origin is appended.
        let seeds = seed_points(&[(1.0, 2.0)], 2);
        assert_eq!(seeds, vec![vec![1.0], vec![2.0], vec![0.0]]);
    }

    #[test]
    fn generation_is_deterministic() {
        let a = seed_points(&[(-2.0, 3.0), (-1.0, 1.0)], 4);
        let b = seed_points(&[(-2.0, 3.0), (-1.0, 1.0)], 4);
        assert_eq!(a, b);
    }
}
