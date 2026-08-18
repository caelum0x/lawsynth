/// Returns a deterministic FNV-1a hash for content-addressing small IR nodes.
///
/// This is not a cryptographic checksum. Bundle integrity must use SHA-256 when
/// the bundle crate is introduced; this hash is intentionally cheap and stable.
pub fn stable_hash(bytes: impl AsRef<[u8]>) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    bytes.as_ref().iter().fold(OFFSET, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(PRIME))
}

#[cfg(test)]
mod tests {
    use super::stable_hash;

    #[test]
    fn stable_hash_is_repeatable() {
        assert_eq!(stable_hash("lawsynth"), stable_hash("lawsynth"));
        assert_ne!(stable_hash("lawsynth"), stable_hash("law synth"));
    }
}
