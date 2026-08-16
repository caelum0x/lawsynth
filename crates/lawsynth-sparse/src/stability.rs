use lawsynth_core::Seed;

use crate::{RegressionProblem, SparseConfig, SparseError, stlsq};

/// Deterministic bootstrap controls for empirical feature-selection stability.
#[derive(Clone, Debug, PartialEq)]
pub struct StabilityConfig {
    pub replicates: usize,
    pub sample_fraction: f64,
    pub seed: Seed,
}

impl Default for StabilityConfig {
    fn default() -> Self {
        Self {
            replicates: 100,
            sample_fraction: 0.75,
            seed: Seed::default(),
        }
    }
}

/// Empirical feature-selection frequencies across bootstrap resamples.
#[derive(Clone, Debug, PartialEq)]
pub struct StabilitySelection {
    pub selections: Vec<usize>,
    pub frequencies: Vec<f64>,
}

/// Measures how often STLSQ retains each feature over bootstrap resamples.
pub fn stability_selection(
    problem: &RegressionProblem,
    sparse: &SparseConfig,
    config: &StabilityConfig,
) -> Result<StabilitySelection, SparseError> {
    if config.replicates == 0
        || !config.sample_fraction.is_finite()
        || !(0.0 < config.sample_fraction && config.sample_fraction <= 1.0)
    {
        return Err(SparseError::InvalidConfig);
    }
    let sample_size = ((problem.rows.len() as f64 * config.sample_fraction).ceil() as usize).max(1);
    let mut selections = vec![0; problem.features()];
    let mut rng = config.seed.rng();
    for _ in 0..config.replicates {
        let indices = (0..sample_size)
            .map(|_| sample_index(&mut rng, problem.rows.len()))
            .collect::<Vec<_>>();
        let sampled = RegressionProblem::new(
            indices
                .iter()
                .map(|index| problem.rows[*index].clone())
                .collect(),
            indices.iter().map(|index| problem.target[*index]).collect(),
        )?;
        let solution = stlsq(&sampled, sparse)?;
        for (count, coefficient) in selections.iter_mut().zip(solution.coefficients) {
            if coefficient.abs() >= sparse.threshold {
                *count += 1;
            }
        }
    }
    Ok(StabilitySelection {
        frequencies: selections
            .iter()
            .map(|count| *count as f64 / config.replicates as f64)
            .collect(),
        selections,
    })
}

fn sample_index(rng: &mut lawsynth_core::DeterministicRng, upper_bound: usize) -> usize {
    let bound = upper_bound as u64;
    let zone = u64::MAX - u64::MAX % bound;
    loop {
        let value = rng.next_u64();
        if value < zone {
            return (value % bound) as usize;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_selection_is_reproducible() {
        let problem = RegressionProblem::new(
            vec![
                vec![1.0, 0.0],
                vec![1.0, 1.0],
                vec![1.0, 2.0],
                vec![1.0, 3.0],
                vec![1.0, 4.0],
                vec![1.0, 5.0],
            ],
            vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0],
        )
        .unwrap();
        let sparse = SparseConfig {
            threshold: 0.1,
            ridge: 1e-3,
            ..Default::default()
        };
        let config = StabilityConfig {
            replicates: 20,
            sample_fraction: 1.0,
            seed: Seed::new(44),
        };
        let first = stability_selection(&problem, &sparse, &config).unwrap();
        let second = stability_selection(&problem, &sparse, &config).unwrap();
        assert_eq!(first, second);
        assert!(first.frequencies[1] > 0.9);
    }
}
