/// Computes the SHA-256 content address used by bundles and artifacts.
pub fn sha256(bytes: &[u8]) -> String {
    lawsynth_bundle::sha256_hex(bytes)
}

/// Returns true only for canonical lowercase SHA-256 hexadecimal strings.
pub fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_bundle_sha256_and_requires_canonical_addresses() {
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(is_sha256_hex(&sha256(b"abc")));
        assert!(!is_sha256_hex(&"A".repeat(64)));
    }
}
