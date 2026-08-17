use lawsynth_plugin_api::{Capability, CapabilitySet, PluginManifest};
use lawsynth_plugin_host::{HostConfig, PermissionPolicy, PermissionSet, PluginRegistry};

#[test]
fn registry_requires_host_enablement_and_explicit_grants() {
    let manifest = PluginManifest::parse("id = adapter\nversion = 1.0.0\nkind = wasi\nentrypoint = adapter.wasm\ncapabilities = data.adapter\n").unwrap();
    let mut registry = PluginRegistry::default();
    assert!(registry.register(&HostConfig::default(), manifest.clone()).is_err());
    let config = HostConfig {
        enabled: true,
        policy: PermissionPolicy {
            allowed: PermissionSet(CapabilitySet::new([Capability::DataAdapter])),
            allow_trusted_native: false,
        },
        maximum_limits: Default::default(),
    };
    registry.register(&config, manifest).unwrap();
    assert_eq!(registry.len(), 1);
    assert!(registry.get("adapter").unwrap().permissions.allows(Capability::DataAdapter));
}
