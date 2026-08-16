use crate::checksum::sha256_hex;

/// A shared-key HMAC-SHA256 authentication tag for bundle bytes.
///
/// This is intentionally a MAC, not an asymmetric signature: verification
/// requires the same secret key that created the tag.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleSignature(pub String);

impl BundleSignature {
    pub fn authenticate(secret: &[u8], bytes: &[u8]) -> Self {
        Self(hmac_sha256_hex(secret, bytes))
    }
}

pub fn verify_signature(secret: &[u8], bytes: &[u8], signature: &BundleSignature) -> bool {
    constant_time_eq(
        signature.0.as_bytes(),
        hmac_sha256_hex(secret, bytes).as_bytes(),
    )
}

fn hmac_sha256_hex(secret: &[u8], bytes: &[u8]) -> String {
    const BLOCK: usize = 64;
    let mut key = if secret.len() > BLOCK {
        hex_to_bytes(&sha256_hex(secret))
    } else {
        secret.to_vec()
    };
    key.resize(BLOCK, 0);
    let mut inner = key.iter().map(|byte| byte ^ 0x36).collect::<Vec<_>>();
    inner.extend_from_slice(bytes);
    let inner_hash = hex_to_bytes(&sha256_hex(&inner));
    let mut outer = key.iter().map(|byte| byte ^ 0x5c).collect::<Vec<_>>();
    outer.extend(inner_hash);
    sha256_hex(&outer)
}

fn hex_to_bytes(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&hex[index..index + 2], 16).expect("sha256 output is hexadecimal")
        })
        .collect()
}
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0u8, |difference, (left, right)| difference | (left ^ right))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authentication_rejects_modified_bytes() {
        let tag = BundleSignature::authenticate(b"key", b"world");
        assert!(verify_signature(b"key", b"world", &tag));
        assert!(!verify_signature(b"key", b"other", &tag));
    }
}
