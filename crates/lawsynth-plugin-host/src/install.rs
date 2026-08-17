//! Installed-plugin record and the capability-grant enforcement point.
//!
//! Declaration is not permission. A plugin's manifest *declares* a capability
//! set; at install time the operator *grants* a subset of it. [`InstalledPlugin`]
//! stores that granted subset and is the single point that answers "is this
//! capability available?" — the answer is yes only when the capability is BOTH
//! declared by the manifest AND granted by the operator.
//!
//! ## What is really enforced
//!
//! - **Capability availability** ([`InstalledPlugin::available`]) — real: an
//!   ungranted or undeclared capability is reported unavailable, unconditionally.
//! - **Resource limits** — recorded from the manifest and bounded by the host
//!   maximum; the host meter ([`crate::ResourceMeter`]) enforces output/request
//!   counts. CPU/memory ceilings are advisory until an OS sandbox is linked.
//! - **Trust** — records whether the package's signature verified against the
//!   configured trust set.
//!
//! OS-level process/filesystem isolation is a documented seam and is NOT linked
//! here; this type never claims isolation it does not provide.

use lawsynth_plugin_api::{Capability, CapabilitySet, PluginManifest, ResourceLimits};

use crate::HostError;

/// Whether an installed package's signature verified against the trust set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrustStatus {
    /// Signature verified against a configured trusted signer.
    Trusted(String),
    /// Unsigned, or signed by a signer absent from / not matching the trust set.
    /// Installable only with an explicit `--allow-unverified` acknowledgement.
    Untrusted,
}

impl TrustStatus {
    /// The verified signer id, if trusted.
    pub fn signer(&self) -> Option<&str> {
        match self {
            Self::Trusted(signer) => Some(signer),
            Self::Untrusted => None,
        }
    }
    /// Whether the package is trusted.
    pub fn is_trusted(&self) -> bool {
        matches!(self, Self::Trusted(_))
    }
}

/// A plugin installed into a local plugin directory.
#[derive(Clone, Debug)]
pub struct InstalledPlugin {
    manifest: PluginManifest,
    granted: CapabilitySet,
    trust: TrustStatus,
    package_hash: String,
}

impl InstalledPlugin {
    /// Records an install. The `granted` set MUST be a subset of the manifest's
    /// declared capabilities, otherwise the install is rejected — the operator
    /// cannot grant what the author never declared.
    pub fn new(
        manifest: PluginManifest,
        granted: CapabilitySet,
        trust: TrustStatus,
        package_hash: impl Into<String>,
    ) -> Result<Self, HostError> {
        manifest.validate()?;
        if !granted.is_subset_of(&manifest.capabilities) {
            let offending = granted
                .iter()
                .find(|cap| !manifest.capabilities.contains(*cap))
                .expect("subset check failed");
            return Err(HostError::PermissionDenied(format!(
                "cannot grant undeclared capability {}",
                offending.as_str()
            )));
        }
        Ok(Self { manifest, granted, trust, package_hash: package_hash.into() })
    }

    /// THE ENFORCEMENT POINT. A capability is available only when the manifest
    /// declares it AND the operator granted it. Everything else is denied.
    pub fn available(&self, capability: Capability) -> bool {
        self.manifest.capabilities.contains(capability) && self.granted.contains(capability)
    }

    /// The granted capability subset the host will honor.
    pub fn granted(&self) -> &CapabilitySet {
        &self.granted
    }

    /// The capabilities declared by the manifest.
    pub fn declared(&self) -> &CapabilitySet {
        &self.manifest.capabilities
    }

    /// The recorded (manifest-declared, host-bounded) resource limits.
    pub fn limits(&self) -> ResourceLimits {
        self.manifest.limits
    }

    /// The plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    /// The trust status recorded at install time.
    pub fn trust(&self) -> &TrustStatus {
        &self.trust
    }

    /// The content hash of the installed package.
    pub fn package_hash(&self) -> &str {
        &self.package_hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(caps: &str) -> PluginManifest {
        PluginManifest::parse(&format!(
            "id = demo\nversion = 1.0.0\nkind = wasi\nentrypoint = d.wasm\ncapabilities = {caps}\n"
        ))
        .unwrap()
    }

    #[test]
    fn available_requires_declared_and_granted() {
        let m = manifest("world.validate, dataset.read");
        let granted = CapabilitySet::new([Capability::WorldValidate]);
        let installed =
            InstalledPlugin::new(m, granted, TrustStatus::Trusted("acme".into()), "hash").unwrap();

        // Declared + granted -> available.
        assert!(installed.available(Capability::WorldValidate));
        // Declared but NOT granted -> unavailable (the grant subset is enforced).
        assert!(!installed.available(Capability::ReadDataset));
        // Never declared -> unavailable.
        assert!(!installed.available(Capability::Network));
        assert!(installed.trust().is_trusted());
    }

    #[test]
    fn cannot_grant_undeclared_capability() {
        let m = manifest("world.validate");
        let granted = CapabilitySet::new([Capability::Network]);
        let result = InstalledPlugin::new(m, granted, TrustStatus::Untrusted, "hash");
        assert!(result.is_err());
    }

    #[test]
    fn empty_grant_makes_nothing_available() {
        let m = manifest("world.validate");
        let installed =
            InstalledPlugin::new(m, CapabilitySet::default(), TrustStatus::Untrusted, "h").unwrap();
        assert!(!installed.available(Capability::WorldValidate));
    }
}
