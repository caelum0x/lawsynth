//! Per-state bootstrap coefficient uncertainty for the discovery pipeline.
//!
//! When the opt-in bootstrap is requested, discovery re-fits each state's
//! feature library on many deterministic resamples of the *same* `(Θ, ẋ)` it
//! used for the point fit, and reports — per candidate term — a percentile
//! confidence interval and an inclusion probability. The heavy lifting is the
//! deterministic bootstrap in [`lawsynth_uncertainty`]; this module only wires
//! discovery's per-state fit inputs into it and pairs the result back with the
//! library column names for rendering.
//!
//! **Honesty.** The intervals are *bootstrap percentile approximations*. They do
//! not carry an exact frequentist coverage guarantee, and — because
//! [`lawsynth_uncertainty::bootstrap_coefficients`] refits with STLSQ — they
//! reflect an STLSQ resampling distribution even when the point fit used a
//! different sparse method. With a small `B` the intervals are simply wide, not
//! wrong.

use lawsynth_core::{Identifier, stable_hash};
use lawsynth_sparse::SparseConfig;
use lawsynth_stats::BootstrapConfig;
use lawsynth_uncertainty::{
    BootstrapCoefficientConfig, CoefficientEnsemble, ResampleMode, bootstrap_coefficients,
};

use crate::DiscoveryError;

/// Bootstrap coefficient uncertainty for one state's fitted feature library.
///
/// [`term_names`](Self::term_names) is in library column order and aligned
/// index-for-index with [`ensemble.terms`](CoefficientEnsemble::terms), so a
/// renderer can pair each candidate term with its confidence interval and
/// inclusion probability. Terms that were inconsistently selected across
/// resamples show up honestly as low inclusion rather than being hidden.
#[derive(Clone, Debug, PartialEq)]
pub struct StateCoefficientEnsemble {
    /// State whose derivative law this ensemble quantifies.
    pub state: Identifier,
    /// Candidate library term names, in the exact column order fed to the fit.
    pub term_names: Vec<String>,
    /// Per-term bootstrap summaries plus the raw replicate draws.
    pub ensemble: CoefficientEnsemble,
}

/// Minimum resamples for a two-sided percentile interval. Below this the
/// coefficient bootstrap is silently skipped (rather than erroring), so a caller
/// that requested `--bootstrap 1` for the MSE bootstrap keeps working.
const MIN_RESAMPLES: usize = 2;

/// Runs the deterministic coefficient bootstrap for one state's `(Θ, ẋ)`.
///
/// `theta`/`target` are the exact per-state fit inputs (after any template and
/// dimensional pruning), and `term_names` are their library column names. The
/// bootstrap reuses the state's `sparse` settings, the requested resample count
/// (`bootstrap.replicates`), and the configured `confidence`. Returns `Ok(None)`
/// when fewer than two resamples were requested.
pub(crate) fn bootstrap_state_coefficients(
    state: &Identifier,
    theta: &[Vec<f64>],
    target: &[f64],
    term_names: &[String],
    sparse: &SparseConfig,
    bootstrap: &BootstrapConfig,
    confidence: f64,
) -> Result<Option<StateCoefficientEnsemble>, DiscoveryError> {
    if bootstrap.replicates < MIN_RESAMPLES {
        return Ok(None);
    }
    let config = BootstrapCoefficientConfig {
        resamples: bootstrap.replicates,
        seed: content_seed(state, theta, target, bootstrap.seed),
        confidence,
        mode: ResampleMode::Cases,
        sparse: sparse.clone(),
    };
    let ensemble = bootstrap_coefficients(theta, target, &config)
        .map_err(|error| DiscoveryError::Sparse(error.to_string()))?;
    Ok(Some(StateCoefficientEnsemble {
        state: state.clone(),
        term_names: term_names.to_vec(),
        ensemble,
    }))
}

/// Derives a deterministic per-state root seed from the fit content.
///
/// The seed is the [`stable_hash`] of the state name and the exact bit patterns
/// of `Θ` and `ẋ`, folded with the configured base seed. It never reads a wall
/// clock, so identical inputs and the same base seed always yield the identical
/// ensemble, while different states or different data decorrelate cleanly.
fn content_seed(state: &Identifier, theta: &[Vec<f64>], target: &[f64], base: u64) -> u64 {
    let mut bytes = Vec::with_capacity(
        state.as_str().len() + (theta.len() * theta.first().map_or(0, Vec::len) + target.len()) * 8,
    );
    bytes.extend_from_slice(state.as_str().as_bytes());
    for row in theta {
        for value in row {
            bytes.extend_from_slice(&value.to_bits().to_le_bytes());
        }
    }
    for value in target {
        bytes.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    stable_hash(&bytes) ^ base
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state() -> Identifier {
        Identifier::new("x").unwrap()
    }

    #[test]
    fn content_seed_is_deterministic_and_content_sensitive() {
        let theta = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let target = vec![1.0, 2.0];
        let base = 7;
        let seed = content_seed(&state(), &theta, &target, base);
        assert_eq!(seed, content_seed(&state(), &theta, &target, base));

        let mut perturbed = theta.clone();
        perturbed[0][0] = 1.5;
        assert_ne!(seed, content_seed(&state(), &perturbed, &target, base));

        let other = Identifier::new("y").unwrap();
        assert_ne!(seed, content_seed(&other, &theta, &target, base));
    }

    #[test]
    fn fewer_than_two_resamples_skips_the_bootstrap() {
        let theta = vec![vec![1.0, 0.0], vec![0.0, 1.0], vec![1.0, 1.0]];
        let target = vec![1.0, 0.0, 1.0];
        let names = vec!["a".to_owned(), "b".to_owned()];
        let bootstrap = BootstrapConfig { replicates: 1, block_size: 4, seed: 0 };
        let result = bootstrap_state_coefficients(
            &state(),
            &theta,
            &target,
            &names,
            &SparseConfig::default(),
            &bootstrap,
            0.95,
        )
        .unwrap();
        assert!(result.is_none());
    }
}
