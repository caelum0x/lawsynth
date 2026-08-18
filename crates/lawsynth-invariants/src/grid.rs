use lawsynth_core::Identifier;
use lawsynth_expr::Environment;

use crate::InvariantConfig;

/// Builds the deterministic tensor-product sample grid over `states`.
///
/// Each axis is sampled at `config.resolution` equally-spaced points across the
/// shared box `[sample_lo, sample_hi]`. The returned environments enumerate the
/// full Cartesian product in a fixed, last-axis-fastest (odometer) order, so the
/// grid — and therefore every downstream matrix and SVD — is reproducible.
pub fn sample_grid(states: &[Identifier], config: &InvariantConfig) -> Vec<Environment> {
    let axis = axis_points(config);
    let dimensions = states.len();
    let total = axis.len().checked_pow(dimensions as u32).unwrap_or(0);
    let mut grid = Vec::with_capacity(total);
    for point_index in 0..total {
        let mut environment = Environment::new();
        let mut remainder = point_index;
        // Last axis varies fastest; divide the flat index down the axes.
        for state in states.iter().rev() {
            let coordinate = axis[remainder % axis.len()];
            remainder /= axis.len();
            environment.insert(state.clone(), coordinate);
        }
        grid.push(environment);
    }
    grid
}

/// The equally-spaced sample coordinates along one axis.
fn axis_points(config: &InvariantConfig) -> Vec<f64> {
    let steps = config.resolution;
    let span = config.sample_hi - config.sample_lo;
    (0..steps)
        .map(|index| config.sample_lo + span * (index as f64) / ((steps - 1) as f64))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn produces_a_full_tensor_grid() {
        let states = [id("x"), id("y")];
        let config = InvariantConfig {
            sample_lo: 0.0,
            sample_hi: 1.0,
            resolution: 3,
            ..InvariantConfig::default()
        };
        let grid = sample_grid(&states, &config);
        assert_eq!(grid.len(), 9);
        // First point is the lower corner; the grid covers the box.
        assert_eq!(grid[0][&id("x")], 0.0);
        assert_eq!(grid[0][&id("y")], 0.0);
        assert_eq!(grid[8][&id("x")], 1.0);
        assert_eq!(grid[8][&id("y")], 1.0);
    }

    #[test]
    fn is_bit_identical_across_calls() {
        let states = [id("x"), id("y")];
        let config = InvariantConfig::default();
        let first = sample_grid(&states, &config);
        let second = sample_grid(&states, &config);
        assert_eq!(first, second);
    }
}
