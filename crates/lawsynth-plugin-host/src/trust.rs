//! Keyed-signature creation and a configurable trust set.
//!
//! ## Honesty: what the signature guarantees
//!
//! A package signature is a **keyed HMAC-SHA256 tag over the package hash**,
//! computed with [`lawsynth_bundle`]'s audited HMAC primitive. It is a
//! *symmetric* construction:
//!
//! - Verifying requires the **same secret** that signed. Whoever can verify can
//!   also forge, so this proves integrity and authenticity *relative to a shared
//!   secret* — it is NOT a public-key signature and provides NO non-repudiation.
//! - The trust set therefore stores per-signer **secrets**, not public keys. In
//!   a real deployment those secrets are distributed out of band to the hosts
//!   that should be able to verify a given signer.
//!
//! This is deliberately the strongest guarantee achievable std-only and offline
//! without a public-key crate. The package format reserves a `SIGNATURE` slot,
//! so swapping in Ed25519 later is a localized change.

use std::collections::BTreeMap;

use lawsynth_bundle::{BundleSignature, verify_signature};

use crate::HostError;
use crate::package::PackageSignature;

/// Produces a signature over `package_hash` using `secret`, attributed to
/// `signer`.
pub fn sign_package_hash(secret: &[u8], signer: &str, package_hash: &str) -> PackageSignature {
    let tag = BundleSignature::authenticate(secret, package_hash.as_bytes());
    PackageSignature { signer: signer.to_owned(), tag: tag.0 }
}

/// Verifies `signature` over `package_hash` using `secret` (constant-time).
pub fn verify_with_secret(secret: &[u8], package_hash: &str, signature: &PackageSignature) -> bool {
    verify_signature(secret, package_hash.as_bytes(), &BundleSignature(signature.tag.clone()))
}

/// A configurable trust set mapping signer ids to their shared secret.
///
/// Persisted as plain, diffable text — one `signer\t<hex-secret>` per line;
/// blank lines and `#` comments are ignored.
#[derive(Clone, Debug, Default)]
pub struct TrustStore {
    keys: BTreeMap<String, Vec<u8>>,
}

impl TrustStore {
    /// Parses a trust-keys file. Secrets are lowercase hex.
    pub fn parse(text: &str) -> Result<Self, HostError> {
        let mut keys = BTreeMap::new();
        for raw in text.lines() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (signer, hex) = line.split_once(['\t', ' ']).ok_or_else(|| {
                HostError::Trust(format!("expected `signer <hex-secret>`: {line}"))
            })?;
            let signer = signer.trim();
            let hex = hex.trim();
            if signer.is_empty() {
                return Err(HostError::Trust("trust entry has an empty signer".into()));
            }
            let secret = decode_hex(hex)
                .ok_or_else(|| HostError::Trust(format!("invalid hex secret for {signer:?}")))?;
            if keys.insert(signer.to_owned(), secret).is_some() {
                return Err(HostError::Trust(format!("duplicate trust entry for {signer:?}")));
            }
        }
        Ok(Self { keys })
    }

    /// Whether the store holds no keys.
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    /// The number of trusted signers.
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Verifies `signature` over `package_hash` against the trusted signer's
    /// secret. Returns the verified signer id on success.
    ///
    /// Verification fails (returns `None`) when the signer is unknown to this
    /// trust set or the tag does not match — an untrusted package.
    pub fn verify(&self, signature: &PackageSignature, package_hash: &str) -> Option<String> {
        let secret = self.keys.get(&signature.signer)?;
        if verify_with_secret(secret, package_hash, signature) {
            Some(signature.signer.clone())
        } else {
            None
        }
    }
}

fn decode_hex(hex: &str) -> Option<Vec<u8>> {
    if hex.is_empty() || hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_a_valid_signature() {
        let sig = sign_package_hash(b"topsecret", "acme", "abc123");
        assert!(verify_with_secret(b"topsecret", "abc123", &sig));
        assert!(!verify_with_secret(b"topsecret", "abc124", &sig));
        assert!(!verify_with_secret(b"wrongkey", "abc123", &sig));
    }

    #[test]
    fn trust_store_verifies_known_signer() {
        // secret bytes 0xde 0xad = hex "dead"
        let store = TrustStore::parse("# comment\nacme\tdead\n").unwrap();
        assert_eq!(store.len(), 1);
        let sig = sign_package_hash(&[0xde, 0xad], "acme", "hash");
        assert_eq!(store.verify(&sig, "hash"), Some("acme".to_owned()));
        // Unknown signer -> untrusted.
        let other = sign_package_hash(&[0xde, 0xad], "mallory", "hash");
        assert_eq!(store.verify(&other, "hash"), None);
        // Right signer, wrong hash -> untrusted.
        assert_eq!(store.verify(&sig, "different"), None);
    }

    #[test]
    fn rejects_malformed_hex() {
        assert!(TrustStore::parse("acme\txyz").is_err());
        assert!(TrustStore::parse("acme\tabc").is_err());
    }
}
