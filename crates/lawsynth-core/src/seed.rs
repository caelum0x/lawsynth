use crate::stable_hash;

/// A stable seed used to reproduce stochastic discovery and resampling choices.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Seed(pub u64);

impl Seed {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Produces an independent deterministic child seed for a named operation.
    pub fn derive(self, label: impl AsRef<[u8]>) -> Self {
        let mut bytes = self.0.to_le_bytes().to_vec();
        bytes.extend_from_slice(label.as_ref());
        Self(stable_hash(bytes))
    }

    pub const fn rng(self) -> DeterministicRng {
        DeterministicRng { state: self.0 }
    }
}

/// Small SplitMix64 generator for deterministic internal sampling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    /// Returns a uniformly distributed value in `[0, 1)` using 53 random bits.
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }
}
