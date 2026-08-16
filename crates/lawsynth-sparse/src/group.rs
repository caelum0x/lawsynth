use crate::{
    RegressionProblem, SparseConfig, SparseError, SparseSolution,
    stlsq::{residual_sum_squares, solve_active, validate_config},
};

/// Group thresholding controls. The embedded sparse config governs least
/// squares and the group norm threshold selects whole groups together.
#[derive(Clone, Debug, PartialEq)]
pub struct GroupConfig {
    pub sparse: SparseConfig,
    pub group_threshold: f64,
}

impl Default for GroupConfig {
    fn default() -> Self {
        Self {
            sparse: SparseConfig::default(),
            group_threshold: 0.1,
        }
    }
}

/// Sequentially thresholds feature groups by their coefficient L2 norm.
///
/// `groups` must form a disjoint complete partition of feature indices. This
/// avoids silently leaving features ungoverned by a structural prior.
pub fn group_stlsq(
    problem: &RegressionProblem,
    groups: &[Vec<usize>],
    config: &GroupConfig,
) -> Result<SparseSolution, SparseError> {
    validate_config(&config.sparse)?;
    if !config.group_threshold.is_finite() || config.group_threshold < 0.0 {
        return Err(SparseError::InvalidConfig);
    }
    validate_groups(problem.features(), groups)?;

    let mut active_groups = (0..groups.len()).collect::<Vec<_>>();
    let mut coefficients = vec![0.0; problem.features()];
    for _ in 0..config.sparse.max_iterations {
        if active_groups.is_empty() {
            break;
        }
        let active_features = active_groups
            .iter()
            .flat_map(|group| groups[*group].iter().copied())
            .collect::<Vec<_>>();
        let fitted = solve_active(problem, &active_features, config.sparse.ridge, None)?;
        coefficients.fill(0.0);
        for (feature, coefficient) in active_features.iter().zip(fitted) {
            coefficients[*feature] = coefficient;
        }
        let next = active_groups
            .iter()
            .copied()
            .filter(|group| {
                let norm = groups[*group]
                    .iter()
                    .map(|feature| coefficients[*feature] * coefficients[*feature])
                    .sum::<f64>()
                    .sqrt();
                norm >= config.group_threshold
            })
            .collect::<Vec<_>>();
        if next == active_groups {
            break;
        }
        active_groups = next;
    }
    Ok(SparseSolution {
        residual_sum_squares: residual_sum_squares(problem, &coefficients),
        coefficients,
    })
}

fn validate_groups(features: usize, groups: &[Vec<usize>]) -> Result<(), SparseError> {
    let mut seen = vec![false; features];
    for group in groups {
        if group.is_empty() {
            return Err(SparseError::InvalidGroups);
        }
        for feature in group {
            if *feature >= features || std::mem::replace(&mut seen[*feature], true) {
                return Err(SparseError::InvalidGroups);
            }
        }
    }
    if seen.iter().all(|feature| *feature) {
        Ok(())
    } else {
        Err(SparseError::InvalidGroups)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_or_drops_an_entire_group() {
        let problem = RegressionProblem::new(
            vec![
                vec![0.0, 0.0, 1.0],
                vec![1.0, 2.0, 1.0],
                vec![2.0, 4.0, 1.0],
                vec![3.0, 6.0, 1.0],
            ],
            vec![0.0, 2.0, 4.0, 6.0],
        )
        .unwrap();
        let solution = group_stlsq(
            &problem,
            &[vec![0, 1], vec![2]],
            &GroupConfig {
                sparse: SparseConfig {
                    ridge: 1e-3,
                    ..Default::default()
                },
                group_threshold: 0.1,
            },
        )
        .unwrap();
        assert!(solution.coefficients[0].abs() > 0.1 || solution.coefficients[1].abs() > 0.1);
        assert_eq!(solution.coefficients[2], 0.0);
    }

    #[test]
    fn rejects_overlapping_or_incomplete_groups() {
        let problem = RegressionProblem::new(vec![vec![1.0, 2.0]], vec![1.0]).unwrap();
        assert_eq!(
            group_stlsq(&problem, &[vec![0], vec![0]], &GroupConfig::default()),
            Err(SparseError::InvalidGroups)
        );
    }
}
