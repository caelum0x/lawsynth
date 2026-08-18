//! Seeded, deterministic pseudo-random draws for the Monte-Carlo forecast.
//!
//! This mirrors the SplitMix64 generator used by `lawsynth-uncertainty`'s
//! coefficient bootstrap (identical constants and per-stream seeding rule) so
//! the two crates draw from the same reproducible sequence family. Nothing here
//! ever reads the wall clock: a draw is a pure function of its `(seed, stream)`.

/// The golden-ratio odd constant that advances the SplitMix64 state.
const GOLDEN_GAMMA: u64 = 0x9e37_79b9_7f4a_7c15;

/// A SplitMix64 stream plus a one-slot cache for the paired Box–Muller normal.
pub(crate) struct SplitMix64 {
    state: u64,
    cached_normal: Option<f64>,
}

impl SplitMix64 {
    /// Derives an independent stream for sample `stream` from `(seed, stream)`,
    /// exactly as `lawsynth-uncertainty` seeds each bootstrap replicate: mix the
    /// stream index into the seed, then take one advancing step to decorrelate
    /// adjacent streams before any draw. Because the state depends only on
    /// `(seed, stream)`, every sample is reproducible regardless of the order in
    /// which samples are computed.
    pub(crate) fn seeded(seed: u64, stream: u64) -> Self {
        let base = seed.wrapping_add(stream.wrapping_mul(GOLDEN_GAMMA));
        // One advancing step decorrelates adjacent streams before the first draw,
        // matching how `lawsynth-uncertainty` seeds each bootstrap replicate.
        Self { state: base.wrapping_add(GOLDEN_GAMMA), cached_normal: None }
    }

    /// One SplitMix64 output word.
    pub(crate) fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(GOLDEN_GAMMA);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// A uniform index in `0..len` via rejection sampling (no modulo bias).
    ///
    /// `len` must be non-zero; callers validate a non-empty ensemble first.
    pub(crate) fn next_index(&mut self, len: usize) -> usize {
        let zone = u64::MAX - (u64::MAX % len as u64);
        loop {
            let value = self.next_u64();
            if value < zone {
                return (value % len as u64) as usize;
            }
        }
    }

    /// A uniform double in the open interval `(0, 1)`.
    ///
    /// The top 53 bits give a value in `[0, 1)`; an exact zero is redrawn so the
    /// logarithm in the Box–Muller transform stays finite.
    fn next_open_unit(&mut self) -> f64 {
        loop {
            let bits = self.next_u64() >> 11;
            let value = bits as f64 * (1.0 / (1u64 << 53) as f64);
            if value > 0.0 {
                return value;
            }
        }
    }

    /// A standard normal draw via the Box–Muller transform.
    ///
    /// Box–Muller produces two independent normals per pair of uniforms; the
    /// second is cached and returned on the next call, so no draw is wasted and
    /// the sequence stays deterministic within a stream.
    pub(crate) fn next_standard_normal(&mut self) -> f64 {
        if let Some(value) = self.cached_normal.take() {
            return value;
        }
        let u1 = self.next_open_unit();
        let u2 = self.next_open_unit();
        let radius = (-2.0 * u1.ln()).sqrt();
        let angle = std::f64::consts::TAU * u2;
        self.cached_normal = Some(radius * angle.sin());
        radius * angle.cos()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_depends_only_on_seed_and_index() {
        let mut a = SplitMix64::seeded(7, 3);
        let mut b = SplitMix64::seeded(7, 3);
        assert_eq!(a.next_u64(), b.next_u64());

        let mut c = SplitMix64::seeded(7, 4);
        let mut d = SplitMix64::seeded(8, 3);
        assert_ne!(SplitMix64::seeded(7, 3).next_u64(), c.next_u64());
        assert_ne!(SplitMix64::seeded(7, 3).next_u64(), d.next_u64());
    }

    #[test]
    fn uniform_draws_lie_in_open_unit_interval() {
        let mut rng = SplitMix64::seeded(42, 0);
        for _ in 0..10_000 {
            let value = rng.next_open_unit();
            assert!(value > 0.0 && value < 1.0);
        }
    }

    #[test]
    fn indices_stay_in_range() {
        let mut rng = SplitMix64::seeded(1, 2);
        for _ in 0..10_000 {
            assert!(rng.next_index(5) < 5);
        }
    }

    #[test]
    fn standard_normal_has_expected_moments() {
        let mut rng = SplitMix64::seeded(9, 1);
        let n = 200_000;
        let mut sum = 0.0;
        let mut sum_sq = 0.0;
        for _ in 0..n {
            let z = rng.next_standard_normal();
            sum += z;
            sum_sq += z * z;
        }
        let mean = sum / n as f64;
        let variance = sum_sq / n as f64 - mean * mean;
        assert!(mean.abs() < 0.02, "mean {mean} not near 0");
        assert!((variance - 1.0).abs() < 0.02, "variance {variance} not near 1");
    }
}
