//! Deterministic, seeded measurement noise for estimator simulation.
//!
//! Noise is drawn from the project's SplitMix64 generator
//! ([`lawsynth_core::DeterministicRng`]) and shaped into standard normals by the
//! Box–Muller transform. Seeded from a fixed `u64`, so a noisy run is
//! bit-reproducible; randomness is **never** drawn from the wall clock.

use lawsynth_core::Seed;

use std::f64::consts::TAU;

/// A specification for zero-mean Gaussian measurement noise.
///
/// Each measured output is perturbed by an independent `N(0, std_dev²)` sample.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeasurementNoise {
    /// The seed for the deterministic generator.
    pub seed: u64,
    /// The per-output standard deviation `σ ≥ 0`.
    pub std_dev: f64,
}

impl MeasurementNoise {
    /// A noise spec with the given seed and standard deviation.
    pub fn new(seed: u64, std_dev: f64) -> Self {
        Self { seed, std_dev }
    }
}

/// A running stream of independent standard-normal samples scaled by `std_dev`.
pub(crate) struct GaussianStream {
    rng: lawsynth_core::DeterministicRng,
    std_dev: f64,
}

impl GaussianStream {
    /// Starts a stream from a measurement-noise spec.
    pub(crate) fn new(spec: MeasurementNoise) -> Self {
        Self { rng: Seed::new(spec.seed).rng(), std_dev: spec.std_dev }
    }

    /// One standard-normal sample via Box–Muller (using only the cosine branch,
    /// drawing a fresh pair each time to keep the draw order simple and stable).
    fn normal(&mut self) -> f64 {
        // Guard against `ln(0)`: the uniform can be exactly 0 only when the top
        // 53 bits are all zero. Clamp to the smallest positive normal.
        let u1 = self.rng.next_f64().max(f64::MIN_POSITIVE);
        let u2 = self.rng.next_f64();
        (-2.0 * u1.ln()).sqrt() * (TAU * u2).cos()
    }

    /// A length-`len` noise vector, one independent sample per component.
    pub(crate) fn sample(&mut self, len: usize) -> Vec<f64> {
        (0..len).map(|_| self.std_dev * self.normal()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_seeds_give_identical_streams() {
        let mut a = GaussianStream::new(MeasurementNoise::new(7, 1.0));
        let mut b = GaussianStream::new(MeasurementNoise::new(7, 1.0));
        for _ in 0..64 {
            let x = a.sample(3);
            let y = b.sample(3);
            for (xi, yi) in x.iter().zip(&y) {
                assert_eq!(xi.to_bits(), yi.to_bits());
            }
        }
    }

    #[test]
    fn zero_std_dev_is_noise_free() {
        let mut stream = GaussianStream::new(MeasurementNoise::new(1, 0.0));
        assert_eq!(stream.sample(4), vec![0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn samples_are_finite_and_roughly_standard() {
        let mut stream = GaussianStream::new(MeasurementNoise::new(42, 1.0));
        let count = 20_000;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for _ in 0..count {
            let value = stream.sample(1)[0];
            assert!(value.is_finite());
            sum += value;
            sum_sq += value * value;
        }
        let mean = sum / count as f64;
        let variance = sum_sq / count as f64 - mean * mean;
        assert!(mean.abs() < 0.05, "mean {mean} not near 0");
        assert!((variance - 1.0).abs() < 0.1, "variance {variance} not near 1");
    }
}
