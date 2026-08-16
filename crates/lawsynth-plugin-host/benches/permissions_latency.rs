use lawsynth_plugin_api::{Capability, CapabilitySet, PluginManifest};
use lawsynth_plugin_host::{PermissionPolicy, PermissionSet};
use std::hint::black_box;
use std::time::Instant;

fn main() {
    let policy = PermissionPolicy {
        allowed: PermissionSet(CapabilitySet::new([Capability::Algorithm])),
        allow_trusted_native: false,
    };
    let manifest = PluginManifest::parse(
        "id=algo\nversion=1.0.0\nkind=wasi\nentrypoint=algo.wasm\ncapabilities=algorithm\n",
    )
    .unwrap();
    let start = Instant::now();
    for _ in 0..100_000 {
        black_box(policy.grant(&manifest).unwrap());
    }
    println!("100000 permission decisions in {:?}", start.elapsed());
}
