/// HMAC-SHA-256 bundle authentication using the bundle crate's constant-time verifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleAuthenticator {
    secret: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BundleVerification {
    pub valid: bool,
}

impl BundleAuthenticator {
    pub fn new(secret: impl Into<Vec<u8>>) -> Self {
        Self { secret: secret.into() }
    }

    pub fn authenticate(&self, bytes: &[u8]) -> lawsynth_bundle::BundleSignature {
        lawsynth_bundle::BundleSignature::authenticate(&self.secret, bytes)
    }

    pub fn verify(
        &self,
        bytes: &[u8],
        signature: &lawsynth_bundle::BundleSignature,
    ) -> BundleVerification {
        BundleVerification {
            valid: lawsynth_bundle::verify_signature(&self.secret, bytes, signature),
        }
    }
}
