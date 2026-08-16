use lawsynth_plugin_api::{Capability, PluginKind, PluginManifest};

#[test]
fn parses_and_validates_a_portable_manifest() {
    let manifest = PluginManifest::parse("id = sample-adapter\nversion = 1.2.3\nkind = process\nentrypoint = plugin-bin\ncapabilities = data.adapter,dataset.read\nmax_requests = 4\n").unwrap();
    assert_eq!(manifest.kind, PluginKind::Process);
    assert!(manifest.capabilities.contains(Capability::DataAdapter));
    assert_eq!(manifest.limits.max_requests, 4);
}
#[test]
fn manifest_rejects_unknown_keys_and_unsafe_entrypoints() {
    assert!(
        PluginManifest::parse("id=x\nversion=1.0.0\nkind=wasi\nentrypoint=../bad\nunknown=yes\n")
            .is_err()
    );
}
