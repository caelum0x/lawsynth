//! The Monte-Carlo (ensemble) method: draw parameter vectors from the ensemble,
//! simulate each with the same fixed-step RK4 as the delta method, and take
//! per-time empirical mean and percentile bands across the trajectories.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sensitivity::{SensitivityConfig, forward_sensitivities};

use crate::bands::ForecastBands;
use crate::covariance::{cholesky, lower_triangular_matvec, validate_covariance};
use crate::error::PropagateError;
use crate::rng::SplitMix64;
use crate::stats::percentile;

/// Where the Monte-Carlo parameter draws come from.
///
/// Both variants are deterministic given a seed. The choice reflects an honest
/// modelling assumption: `Gaussian` assumes the coefficients are approximately
/// normal with the given mean and covariance; `Replicates` makes no distributional
/// assumption and simply resamples the empirical bootstrap draws.
pub enum EnsembleSource<'a> {
    /// Sample `θ ~ N(mean, covariance)` via a seeded Box–Muller draw shaped by
    /// the Cholesky factor of `covariance`. `mean` and `covariance` are in
    /// `parameters` order; `covariance` must be `p × p` and positive definite.
    Gaussian { mean: &'a [f64], covariance: &'a [Vec<f64>] },
    /// Resample the raw bootstrap replicate coefficient vectors (shape `[B][p]`)
    /// with replacement — the empirical, assumption-free ensemble.
    Replicates { draws: &'a [Vec<f64>] },
}

/// Propagate parameter uncertainty into forecast bands by seeded Monte-Carlo.
///
/// `samples` parameter vectors are drawn from `source`; each is simulated with
/// the same fixed-step RK4 integrator the delta method uses (via
/// `lawsynth-sensitivity`, reading the state trajectory), and the bands are the
/// per-time empirical mean, unbiased variance, and two-sided `confidence`
/// percentile interval across the ensemble.
///
/// Determinism is total: sample `m` draws from a SplitMix64 stream seeded by
/// `(seed, m)`, so the ensemble is bit-reproducible and independent of the order
/// in which samples are computed.
///
/// # Errors
///
/// - [`PropagateError::ZeroSamples`] if `samples == 0`.
/// - [`PropagateError::InvalidConfidence`] if `confidence ∉ (0, 1)`.
/// - [`PropagateError::EmptyEnsemble`] / [`PropagateError::ReplicateDimensionMismatch`]
///   for a malformed `Replicates` source.
/// - covariance-shape, [`PropagateError::NonFiniteValue`], and
///   [`PropagateError::NotPositiveSemiDefinite`] errors for a malformed or
///   indefinite `Gaussian` source.
/// - [`PropagateError::Sensitivity`] for any failure of the underlying
///   simulation at a drawn parameter vector.
#[allow(clippy::too_many_arguments)] // The propagation contract fixes this surface.
pub fn monte_carlo_forecast(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    parameters: &[Identifier],
    initial: &[f64],
    source: EnsembleSource<'_>,
    config: &SensitivityConfig,
    samples: usize,
    seed: u64,
    confidence: f64,
) -> Result<ForecastBands, PropagateError> {
    if samples == 0 {
        return Err(PropagateError::ZeroSamples);
    }
    if !confidence.is_finite() || confidence <= 0.0 || confidence >= 1.0 {
        return Err(PropagateError::InvalidConfidence(confidence));
    }
    let drawer = Drawer::new(&source, parameters.len())?;

    // Per-sample state trajectories, shape [sample][step][state].
    let mut ensemble: Vec<Vec<Vec<f64>>> = Vec::with_capacity(samples);
    let mut grid: Option<(Vec<f64>, Vec<Identifier>)> = None;
    for sample in 0..samples {
        let mut rng = SplitMix64::seeded(seed, sample as u64);
        let theta = drawer.draw(&mut rng);
        let trajectory =
            forward_sensitivities(fields, states, parameters, initial, &theta, config)?;
        if grid.is_none() {
            grid = Some((trajectory.times().to_vec(), trajectory.states().to_vec()));
        }
        let series: Vec<Vec<f64>> = (0..trajectory.sample_count())
            .map(|step| trajectory.state_at(step).unwrap().to_vec())
            .collect();
        ensemble.push(series);
    }

    let (times, state_ids) = grid.expect("at least one sample was simulated");
    Ok(aggregate(&ensemble, times, state_ids, confidence))
}

/// Turns the per-sample trajectories into mean / variance / percentile bands.
fn aggregate(
    ensemble: &[Vec<Vec<f64>>],
    times: Vec<f64>,
    state_ids: Vec<Identifier>,
    confidence: f64,
) -> ForecastBands {
    let sample_count = times.len();
    let dimension = state_ids.len();
    let m = ensemble.len();
    let tail = (1.0 - confidence) / 2.0;

    let mut mean = vec![vec![0.0; sample_count]; dimension];
    let mut variance = vec![vec![0.0; sample_count]; dimension];
    let mut lower = vec![vec![0.0; sample_count]; dimension];
    let mut upper = vec![vec![0.0; sample_count]; dimension];

    for state in 0..dimension {
        for step in 0..sample_count {
            let values: Vec<f64> = ensemble.iter().map(|series| series[step][state]).collect();
            let sum: f64 = values.iter().sum();
            let mu = sum / m as f64;
            let var = if m >= 2 {
                values.iter().map(|value| (value - mu) * (value - mu)).sum::<f64>() / (m - 1) as f64
            } else {
                0.0
            };
            mean[state][step] = mu;
            variance[state][step] = var;
            lower[state][step] = percentile(&values, tail);
            upper[state][step] = percentile(&values, 1.0 - tail);
        }
    }

    ForecastBands::new(times, state_ids, mean, variance, lower, upper)
}

/// A validated parameter-vector generator for one Monte-Carlo run.
enum Drawer<'a> {
    Gaussian { mean: &'a [f64], factor: Vec<Vec<f64>> },
    Replicates { draws: &'a [Vec<f64>] },
}

impl<'a> Drawer<'a> {
    fn new(source: &EnsembleSource<'a>, parameters: usize) -> Result<Self, PropagateError> {
        match source {
            EnsembleSource::Gaussian { mean, covariance } => {
                if mean.len() != parameters {
                    return Err(PropagateError::CovarianceDimensionMismatch {
                        expected: parameters,
                        actual: mean.len(),
                    });
                }
                if mean.iter().any(|value| !value.is_finite()) {
                    return Err(PropagateError::NonFiniteValue);
                }
                validate_covariance(covariance, parameters)?;
                let factor = cholesky(covariance)?;
                Ok(Self::Gaussian { mean, factor })
            }
            EnsembleSource::Replicates { draws } => {
                if draws.is_empty() {
                    return Err(PropagateError::EmptyEnsemble);
                }
                for draw in *draws {
                    if draw.len() != parameters {
                        return Err(PropagateError::ReplicateDimensionMismatch {
                            expected: parameters,
                            actual: draw.len(),
                        });
                    }
                    if draw.iter().any(|value| !value.is_finite()) {
                        return Err(PropagateError::NonFiniteValue);
                    }
                }
                Ok(Self::Replicates { draws })
            }
        }
    }

    /// One parameter vector from this run's stream.
    fn draw(&self, rng: &mut SplitMix64) -> Vec<f64> {
        match self {
            Self::Gaussian { mean, factor } => {
                let standard: Vec<f64> =
                    (0..mean.len()).map(|_| rng.next_standard_normal()).collect();
                let correlated = lower_triangular_matvec(factor, &standard);
                mean.iter().zip(&correlated).map(|(m, delta)| m + delta).collect()
            }
            Self::Replicates { draws } => {
                let index = rng.next_index(draws.len());
                draws[index].clone()
            }
        }
    }
}
