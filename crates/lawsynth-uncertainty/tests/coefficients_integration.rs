//! Honest statistical tests for the deterministic coefficient bootstrap.
//!
//! Data is generated from a KNOWN sparse law with deterministic, seeded
//! Gaussian noise (SplitMix64 + Box–Muller), so every assertion is
//! bit-reproducible and never draws randomness from the wall clock.

use lawsynth_sparse::SparseConfig;
use lawsynth_uncertainty::{
    BootstrapCoefficientConfig, ResampleMode, UncertaintyError, bootstrap_coefficients,
};

/// Deterministic standard-normal source via SplitMix64 and Box–Muller.
struct Noise {
    state: u64,
}

impl Noise {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_uniform(&mut self) -> f64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        // Top 53 bits, centred to avoid ln(0).
        ((z >> 11) as f64 + 0.5) / (1_u64 << 53) as f64
    }

    fn normal(&mut self) -> f64 {
        let u1 = self.next_uniform();
        let u2 = self.next_uniform();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// The ground-truth sparse law: `y = 2·x0 − 3·x2` with `x1`, `x3` spurious.
const TRUE_COEFFICIENTS: [f64; 4] = [2.0, 0.0, -3.0, 0.0];

/// Builds a deterministic design matrix and noisy target from the known law.
fn known_law(observations: usize, sigma: f64, seed: u64) -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut source = Noise::new(seed);
    let mut theta = Vec::with_capacity(observations);
    let mut target = Vec::with_capacity(observations);
    for _ in 0..observations {
        // Independent, well-scaled features in roughly [-1, 1].
        let row: Vec<f64> = (0..4).map(|_| 2.0 * source.next_uniform() - 1.0).collect();
        let clean: f64 = row.iter().zip(TRUE_COEFFICIENTS).map(|(x, c)| x * c).sum();
        target.push(clean + sigma * source.normal());
        theta.push(row);
    }
    (theta, target)
}

fn config(resamples: usize, seed: u64, mode: ResampleMode) -> BootstrapCoefficientConfig {
    BootstrapCoefficientConfig {
        resamples,
        seed,
        confidence: 0.95,
        mode,
        sparse: SparseConfig { threshold: 0.3, max_iterations: 20, ridge: 1e-9 },
    }
}

#[test]
fn true_terms_have_high_inclusion_and_covering_intervals() {
    let (theta, target) = known_law(240, 0.1, 1);
    let ensemble =
        bootstrap_coefficients(&theta, &target, &config(300, 7, ResampleMode::Cases)).unwrap();

    let t0 = &ensemble.terms[0];
    let t2 = &ensemble.terms[2];
    // True terms survive sparsity in essentially every resample.
    assert!(t0.inclusion_probability > 0.95, "x0 inclusion {}", t0.inclusion_probability);
    assert!(t2.inclusion_probability > 0.95, "x2 inclusion {}", t2.inclusion_probability);
    // Their intervals cover the true coefficients.
    assert!(t0.lower <= 2.0 && 2.0 <= t0.upper, "x0 CI [{}, {}]", t0.lower, t0.upper);
    assert!(t2.lower <= -3.0 && -3.0 <= t2.upper, "x2 CI [{}, {}]", t2.lower, t2.upper);
}

#[test]
fn spurious_terms_have_low_inclusion_and_intervals_near_zero() {
    let (theta, target) = known_law(240, 0.1, 1);
    let ensemble =
        bootstrap_coefficients(&theta, &target, &config(300, 7, ResampleMode::Cases)).unwrap();

    for &column in &[1usize, 3usize] {
        let term = &ensemble.terms[column];
        assert!(
            term.inclusion_probability < 0.2,
            "spurious x{column} inclusion {}",
            term.inclusion_probability
        );
        // The interval brackets zero (usually exactly [0, 0]).
        assert!(
            term.lower <= 0.0 && 0.0 <= term.upper,
            "spurious x{column} CI [{}, {}]",
            term.lower,
            term.upper
        );
        assert!(term.mean.abs() < 0.3, "spurious x{column} mean {}", term.mean);
    }
}

#[test]
fn interval_width_shrinks_as_sample_size_grows() {
    let width_at = |observations: usize| {
        let (theta, target) = known_law(observations, 0.2, 2);
        let ensemble =
            bootstrap_coefficients(&theta, &target, &config(300, 11, ResampleMode::Cases)).unwrap();
        let term = &ensemble.terms[0];
        term.upper - term.lower
    };
    let small = width_at(60);
    let large = width_at(600);
    // A consistent estimator: more data tightens the interval substantially.
    assert!(large < small, "expected shrinking CI: n=60 -> {small}, n=600 -> {large}");
    assert!(large < 0.6 * small, "expected clear shrinkage: {small} -> {large}");
}

#[test]
fn interval_width_shrinks_as_noise_decreases() {
    let width_at = |sigma: f64| {
        let (theta, target) = known_law(240, sigma, 3);
        let ensemble =
            bootstrap_coefficients(&theta, &target, &config(300, 5, ResampleMode::Cases)).unwrap();
        let term = &ensemble.terms[0];
        term.upper - term.lower
    };
    let noisy = width_at(0.4);
    let clean = width_at(0.05);
    assert!(clean < noisy, "expected tighter CI at lower noise: {noisy} -> {clean}");
}

#[test]
fn ensemble_is_bit_identical_across_runs() {
    let (theta, target) = known_law(120, 0.15, 4);
    let cfg = config(150, 99, ResampleMode::Cases);
    let first = bootstrap_coefficients(&theta, &target, &cfg).unwrap();
    let second = bootstrap_coefficients(&theta, &target, &cfg).unwrap();

    assert_eq!(first.replicates.len(), second.replicates.len());
    for (a, b) in first.replicates.iter().zip(&second.replicates) {
        for (x, y) in a.iter().zip(b) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
    }
    // Aggregated statistics are bit-identical too.
    for (a, b) in first.terms.iter().zip(&second.terms) {
        assert_eq!(a.mean.to_bits(), b.mean.to_bits());
        assert_eq!(a.standard_error.to_bits(), b.standard_error.to_bits());
        assert_eq!(a.lower.to_bits(), b.lower.to_bits());
        assert_eq!(a.upper.to_bits(), b.upper.to_bits());
        assert_eq!(a.inclusion_probability.to_bits(), b.inclusion_probability.to_bits());
    }
}

#[test]
fn replicate_draws_are_independent_of_the_number_of_resamples() {
    let (theta, target) = known_law(120, 0.15, 4);
    let small =
        bootstrap_coefficients(&theta, &target, &config(30, 99, ResampleMode::Cases)).unwrap();
    let large =
        bootstrap_coefficients(&theta, &target, &config(90, 99, ResampleMode::Cases)).unwrap();

    // Replicate `b` depends only on `(seed, b)`, so the first 30 draws match.
    assert_eq!(large.replicates.len(), 90);
    for (a, b) in small.replicates.iter().zip(&large.replicates) {
        for (x, y) in a.iter().zip(b) {
            assert_eq!(x.to_bits(), y.to_bits());
        }
    }
}

#[test]
fn residual_bootstrap_also_recovers_the_law() {
    let (theta, target) = known_law(240, 0.1, 1);
    let ensemble =
        bootstrap_coefficients(&theta, &target, &config(300, 7, ResampleMode::Residual)).unwrap();

    assert!(ensemble.terms[0].inclusion_probability > 0.95);
    assert!(ensemble.terms[2].inclusion_probability > 0.95);
    assert!(ensemble.terms[1].inclusion_probability < 0.2);
    assert!(ensemble.terms[3].inclusion_probability < 0.2);
    assert!(ensemble.terms[0].lower <= 2.0 && 2.0 <= ensemble.terms[0].upper);
    assert!(ensemble.terms[2].lower <= -3.0 && -3.0 <= ensemble.terms[2].upper);
}

#[test]
fn zero_resamples_are_rejected() {
    let (theta, target) = known_law(20, 0.1, 1);
    assert_eq!(
        bootstrap_coefficients(&theta, &target, &config(0, 7, ResampleMode::Cases)).unwrap_err(),
        UncertaintyError::InvalidBootstrapConfig
    );
}

#[test]
fn invalid_confidence_is_rejected() {
    let (theta, target) = known_law(20, 0.1, 1);
    let mut cfg = config(50, 7, ResampleMode::Cases);
    cfg.confidence = 1.5;
    assert_eq!(
        bootstrap_coefficients(&theta, &target, &cfg).unwrap_err(),
        UncertaintyError::InvalidConfidence(1.5)
    );
}

#[test]
fn dimension_mismatch_is_rejected() {
    let theta = vec![vec![1.0, 2.0], vec![3.0, 4.0], vec![5.0, 6.0]];
    let target = vec![1.0, 2.0];
    assert_eq!(
        bootstrap_coefficients(&theta, &target, &config(50, 7, ResampleMode::Cases)).unwrap_err(),
        UncertaintyError::DimensionMismatch { expected: 3, actual: 2 }
    );
}

#[test]
fn degenerate_all_zero_target_yields_zero_inclusion() {
    let (theta, _) = known_law(80, 0.1, 1);
    let target = vec![0.0; theta.len()];
    let ensemble =
        bootstrap_coefficients(&theta, &target, &config(50, 7, ResampleMode::Cases)).unwrap();

    for term in &ensemble.terms {
        assert_eq!(term.inclusion_probability, 0.0);
        assert_eq!(term.lower, 0.0);
        assert_eq!(term.upper, 0.0);
        assert_eq!(term.mean, 0.0);
        assert_eq!(term.standard_error, 0.0);
    }
}

#[test]
fn non_finite_input_is_rejected() {
    let mut theta = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    theta[0][1] = f64::NAN;
    let target = vec![1.0, 2.0];
    assert_eq!(
        bootstrap_coefficients(&theta, &target, &config(50, 7, ResampleMode::Cases)).unwrap_err(),
        UncertaintyError::NonFiniteValue
    );
}
