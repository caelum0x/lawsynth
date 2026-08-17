//! End-to-end coverage for the `lawsynth plugin` marketplace flow:
//! pack -> sign -> install (with grant + trust) -> verify -> tamper -> fail,
//! plus the unsigned/untrusted path. Runs against a temporary plugin directory
//! so it never touches the real `~/.lawsynth`.

use std::fs;
use std::path::{Path, PathBuf};

/// A unique scratch directory for one test (no wall clock; keyed by pid + name).
fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("lawsynth_p8_{}_{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, contents).unwrap();
}

/// A minimal but valid plugin source directory.
fn sample_plugin(root: &Path) -> PathBuf {
    let dir = root.join("plugin-src");
    write(
        &dir.join("plugin.manifest"),
        "id = demo-adapter\nversion = 1.2.0\nkind = wasi\nentrypoint = adapter.wasm\ncapabilities = data.adapter, dataset.read\n",
    );
    write(&dir.join("adapter.wasm"), "\0asm fake artifact bytes");
    write(&dir.join("README.md"), "# demo-adapter\n");
    dir
}

fn run(args: &[&str]) -> Result<String, String> {
    let owned: Vec<String> = args.iter().map(|arg| (*arg).to_owned()).collect();
    lawsynth_cli::run(&owned)
}

#[test]
fn pack_sign_install_verify_and_tamper_detection() {
    let root = scratch("full");
    let src = sample_plugin(&root);
    let plugins = root.join("plugins");
    let pkg = root.join("demo.lsplugin");
    let keyfile = root.join("acme.key");
    let trust = root.join("trust.keys");
    write(&keyfile, "signer = acme\nsecret = 00ff00ff\n");
    write(&trust, "acme\t00ff00ff\n");

    // pack + sign
    let packed = run(&[
        "plugin",
        "pack",
        src.to_str().unwrap(),
        "--output",
        pkg.to_str().unwrap(),
        "--sign",
        keyfile.to_str().unwrap(),
    ])
    .unwrap();
    assert!(packed.contains("id:       demo-adapter"));
    assert!(packed.contains("signed:   acme"));
    assert!(pkg.is_file());

    // A signed package whose signer is NOT trusted must be refused without the flag.
    let empty_trust = root.join("empty.keys");
    write(&empty_trust, "# no signers\n");
    let refused = run(&[
        "plugin",
        "install",
        pkg.to_str().unwrap(),
        "--dir",
        plugins.to_str().unwrap(),
        "--grant",
        "data.adapter",
        "--trust",
        empty_trust.to_str().unwrap(),
    ])
    .unwrap_err();
    assert!(refused.contains("untrusted"), "unexpected: {refused}");

    // install with the trust set + a partial capability grant
    let installed = run(&[
        "plugin",
        "install",
        pkg.to_str().unwrap(),
        "--dir",
        plugins.to_str().unwrap(),
        "--grant",
        "data.adapter",
        "--trust",
        trust.to_str().unwrap(),
    ])
    .unwrap();
    assert!(installed.contains("trusted (signer: acme)"));
    assert!(installed.contains("granted:  data.adapter"));

    // list shows it as trusted
    let listing = run(&["plugin", "list", "--dir", plugins.to_str().unwrap()]).unwrap();
    assert!(listing.contains("demo-adapter"));
    assert!(listing.contains("[trusted]"));

    // verify: the ungranted-but-declared capability is DENIED (enforcement point)
    let verified = run(&[
        "plugin",
        "verify",
        "demo-adapter",
        "--dir",
        plugins.to_str().unwrap(),
        "--trust",
        trust.to_str().unwrap(),
    ])
    .unwrap();
    assert!(verified.contains("VERIFY OK"));
    let decisions: Vec<&str> = verified.lines().collect();
    assert!(
        decisions
            .iter()
            .any(|line| line.contains("data.adapter") && line.trim_end().ends_with("available")),
        "expected data.adapter available in:\n{verified}"
    );
    assert!(
        decisions
            .iter()
            .any(|line| line.contains("dataset.read") && line.contains("DENIED (not granted)")),
        "expected dataset.read DENIED in:\n{verified}"
    );

    // non-destructive: same id without --force is refused
    let dup = run(&[
        "plugin",
        "install",
        pkg.to_str().unwrap(),
        "--dir",
        plugins.to_str().unwrap(),
        "--trust",
        trust.to_str().unwrap(),
    ])
    .unwrap_err();
    assert!(dup.contains("refusing to replace"));

    // tamper one payload byte of the stored package -> verify FAILS
    let stored = plugins.join("demo-adapter").join("package.lsplugin");
    let mut bytes = fs::read(&stored).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;
    fs::write(&stored, &bytes).unwrap();
    let failure =
        run(&["plugin", "verify", "demo-adapter", "--dir", plugins.to_str().unwrap()]).unwrap_err();
    assert!(failure.contains("VERIFY FAILED"), "unexpected: {failure}");

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unsigned_package_requires_allow_unverified() {
    let root = scratch("unsigned");
    let src = sample_plugin(&root);
    let plugins = root.join("plugins");
    let pkg = root.join("demo.lsplugin");

    run(&["plugin", "pack", src.to_str().unwrap(), "--output", pkg.to_str().unwrap()]).unwrap();

    // Without the flag: refused.
    let refused =
        run(&["plugin", "install", pkg.to_str().unwrap(), "--dir", plugins.to_str().unwrap()])
            .unwrap_err();
    assert!(refused.contains("unsigned"));

    // With the flag: installed but untrusted.
    let installed = run(&[
        "plugin",
        "install",
        pkg.to_str().unwrap(),
        "--dir",
        plugins.to_str().unwrap(),
        "--allow-unverified",
    ])
    .unwrap();
    assert!(installed.contains("UNTRUSTED"));

    let listing = run(&["plugin", "list", "--dir", plugins.to_str().unwrap()]).unwrap();
    assert!(listing.contains("[untrusted]"));

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn registry_index_is_plain_and_mirrorable() {
    let root = scratch("registry");
    let src = sample_plugin(&root);
    let pkg = root.join("demo.lsplugin");
    let plugins = root.join("plugins");
    run(&["plugin", "pack", src.to_str().unwrap(), "--output", pkg.to_str().unwrap()]).unwrap();

    run(&[
        "plugin",
        "registry",
        "add",
        pkg.to_str().unwrap(),
        "--dir",
        plugins.to_str().unwrap(),
        "--location",
        "mirror/demo-1.2.0.lsplugin",
    ])
    .unwrap();

    let index = fs::read_to_string(plugins.join("registry.tsv")).unwrap();
    assert!(index.starts_with("id\tversion\tpackage_hash\tlocation\tsigner\n"));
    assert!(index.contains("demo-adapter\t1.2.0\t"));
    assert!(index.contains("mirror/demo-1.2.0.lsplugin"));

    let _ = fs::remove_dir_all(&root);
}
