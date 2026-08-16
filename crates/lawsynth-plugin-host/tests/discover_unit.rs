use lawsynth_plugin_host::discover_manifests;
use std::fs;

#[test]
fn discovery_reads_only_valid_child_manifests() {
    let root =
        std::env::temp_dir().join(format!("lawsynth-plugin-discover-{}", std::process::id()));
    let dir = root.join("one");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("plugin.manifest"),
        "id=one\nversion=1.0.0\nkind=wasi\nentrypoint=one.wasm\n",
    )
    .unwrap();
    assert_eq!(discover_manifests(&root).unwrap()[0].id, "one");
    fs::remove_dir_all(root).unwrap();
}
