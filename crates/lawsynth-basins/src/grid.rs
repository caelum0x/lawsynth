//! Deterministic initial-condition grid over the search box.
//!
//! Unlike the Newton seed lattice in `lawsynth-stability`, the basin grid is the
//! set of initial conditions whose fate we classify, so it is *exactly* the
//! even lattice over the box — no origin is appended. For `resolution` samples
//! per axis and `n` states the grid has `resolution^n` points, enumerated in a
//! fixed row-major order (the first axis varies slowest), which is the order the
//! reported `grid_labels` follow.

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

/// Builds the deterministic initial-condition grid over `search_box`.
///
/// The points are the Cartesian product of the per-axis lattices, enumerated in
/// row-major order (first axis outermost / slowest). This ordering is stable and
/// is the exact order of the labels in [`crate::BasinReport::grid_labels`].
pub(crate) fn initial_conditions(search_box: &[(f64, f64)], resolution: usize) -> Vec<Vec<f64>> {
    let axes: Vec<Vec<f64>> =
        search_box.iter().map(|&(lower, upper)| axis_samples(lower, upper, resolution)).collect();

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
        assert_eq!(axis_samples(-2.0, 2.0, 5), vec![-2.0, -1.0, 0.0, 1.0, 2.0]);
    }

    #[test]
    fn grid_has_resolution_pow_dim_points_in_row_major_order() {
        let grid = initial_conditions(&[(-1.0, 1.0), (0.0, 1.0)], 2);
        assert_eq!(grid, vec![vec![-1.0, 0.0], vec![-1.0, 1.0], vec![1.0, 0.0], vec![1.0, 1.0],]);
    }

    #[test]
    fn generation_is_deterministic() {
        let a = initial_conditions(&[(-2.0, 3.0), (-1.0, 1.0)], 4);
        let b = initial_conditions(&[(-2.0, 3.0), (-1.0, 1.0)], 4);
        assert_eq!(a, b);
    }
}
