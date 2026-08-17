//! `lawsynth plugin <pack|install|list|verify|remove|registry>` — a
//! deterministic, offline plugin marketplace against a local directory.
//!
//! Implements the P8 boundary spec (`specs/plugin-marketplace/README.md`) on top
//! of the `lawsynth-plugin-host` package/trust/install primitives:
//!
//! - **pack** — bundle a plugin directory (its `plugin.manifest`/`plugin.toml`
//!   plus artifact files) into a self-describing, SHA-256-checksummed
//!   `.lsplugin` package, optionally signing over the package hash.
//! - **install** — verify checksums + signature, record a granted capability
//!   subset, and write the package + metadata into a local plugin dir. Refuses a
//!   different version without `--force`; untrusted packages need
//!   `--allow-unverified`.
//! - **list / verify / remove** — inspect, re-verify (checksums + signature),
//!   and uninstall.
//! - **registry add / list** — maintain a plain, diffable, mirrorable index
//!   (`id + version -> package hash + location + signer`).
//!
//! Everything is offline and clock-free; the `.lsplugin` bytes and every text
//! artifact are byte-deterministic.
//!
//! ## Honest scope of enforcement
//!
//! Real: checksum + signature verification, and the capability-grant subset
//! (`lawsynth_plugin_host::InstalledPlugin::available`). Recorded and bounded:
//! resource limits. NOT provided here: OS-level process/filesystem isolation —
//! a documented host seam. The "signature" is a keyed HMAC over the package hash
//! (shared-secret authenticity, not public-key non-repudiation) — see
//! `lawsynth-plugin-host`'s trust module.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_plugin_api::{Capability, CapabilitySet, PluginManifest};
use lawsynth_plugin_host::{
    InstalledPlugin, TrustStatus, TrustStore, build_checksums, pack as pack_bytes, package_hash_of,
    sign_package_hash, unpack,
};

const REGISTRY_FILE: &str = "registry.tsv";
const REGISTRY_HEADER: &str = "id\tversion\tpackage_hash\tlocation\tsigner";
const DEFAULT_TRUST_FILE: &str = "trust.keys";
const INSTALL_FILE: &str = "install.toml";
const PACKAGE_FILE: &str = "package.lsplugin";
/// Directory components never packed (build output / VCS / caches).
const EXCLUDED_DIRS: [&str; 5] = ["target", ".git", "node_modules", "__pycache__", ".pytest_cache"];

/// Help text for `lawsynth plugin`.
pub fn help() -> String {
    "lawsynth plugin <pack|install|list|verify|remove|registry> ...\n\n\
  plugin pack DIR --output PKG.lsplugin [--sign KEYFILE]\n\
      Bundle DIR's plugin.manifest (or plugin.toml) and artifact files into a\n\
      self-describing, SHA-256-checksummed .lsplugin. --sign adds a keyed-HMAC\n\
      signature over the package hash (see KEYFILE format below).\n\
  plugin install PKG.lsplugin [--grant CAP1,CAP2] [--allow-unverified] [--force]\n\
                 [--dir DIR] [--trust KEYSFILE]\n\
      Verify checksums + signature, grant a subset of the declared capabilities,\n\
      and install into DIR (default ~/.lawsynth/plugins). Refuses a different\n\
      installed version without --force. Unsigned/unverifiable packages install\n\
      only with --allow-unverified and are marked untrusted.\n\
  plugin list [--dir DIR]\n\
      List installed plugins with version, trust, and granted capabilities.\n\
  plugin verify ID [--dir DIR] [--trust KEYSFILE]\n\
      Re-verify an installed plugin's checksums and signature; show trusted +\n\
      granted (enforced) capabilities. Non-zero exit on failure.\n\
  plugin remove ID [--dir DIR]\n\
      Uninstall a plugin.\n\
  plugin registry add PKG.lsplugin [--dir DIR] [--registry FILE] [--location LOC]\n\
  plugin registry list [--dir DIR] [--registry FILE]\n\
      Maintain a plain, diffable, mirrorable index of id+version -> package hash\n\
      + location + signer.\n\n\
KEYFILE (for --sign) is `key = value` lines:\n\
    signer = acme-labs\n\
    secret = <lowercase hex bytes>\n\
KEYSFILE (trust set) is one `SIGNER<TAB><hex-secret>` per line (# comments ok).\n\n\
Honesty: the signature is a keyed HMAC-SHA256 over the package hash — it proves\n\
integrity + authenticity relative to a SHARED SECRET (whoever can verify can\n\
forge), not public-key authorship. Real enforcement: checksum + signature\n\
verification and capability grants. Resource limits are recorded/bounded. OS\n\
process/filesystem isolation is a documented, unlinked host seam."
        .to_owned()
}

/// Runs the `plugin` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(subcommand) = arguments.first().map(String::as_str) else {
        return Err(help());
    };
    match subcommand {
        "--help" | "-h" | "help" => Ok(help()),
        "pack" => pack_command(&arguments[1..]),
        "install" => install_command(&arguments[1..]),
        "list" => list_command(&arguments[1..]),
        "verify" => verify_command(&arguments[1..]),
        "remove" => remove_command(&arguments[1..]),
        "registry" => registry_command(&arguments[1..]),
        _ => Err(help()),
    }
}

// ---------------------------------------------------------------------------
// pack
// ---------------------------------------------------------------------------

fn pack_command(arguments: &[String]) -> Result<String, String> {
    let mut dir = None;
    let mut output = None;
    let mut sign = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--output" => {
                output = Some(value_of(arguments, index, "--output")?);
                index += 2;
            }
            "--sign" => {
                sign = Some(value_of(arguments, index, "--sign")?);
                index += 2;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => {
                if dir.is_some() {
                    return Err("pack takes a single DIR".to_owned());
                }
                dir = Some(arguments[index].clone());
                index += 1;
            }
        }
    }
    let dir = dir.ok_or("usage: plugin pack DIR --output PKG.lsplugin [--sign KEYFILE]")?;
    let output = output.ok_or("plugin pack requires --output PKG.lsplugin")?;

    let files = collect_package_files(Path::new(&dir))?;
    let manifest = parse_manifest_from(&files)?;

    // Compute the package hash from the checksum manifest, then (optionally) sign
    // it, so the package is packed exactly once.
    let checksums = build_checksums(&files);
    let package_hash = package_hash_of(&checksums);
    let signature = match &sign {
        Some(keyfile) => {
            let (signer, secret) = read_signing_key(keyfile)?;
            Some(sign_package_hash(&secret, &signer, &package_hash))
        }
        None => None,
    };
    let (bytes, packed_hash) =
        pack_bytes(&files, signature.as_ref()).map_err(|error| error.to_string())?;
    debug_assert_eq!(packed_hash, package_hash);

    fs::write(&output, &bytes).map_err(|error| format!("failed to write {output}: {error}"))?;

    let mut out = format!("packed {} -> {output} ({} bytes)\n", dir, bytes.len());
    let _ = writeln!(out, "  id:       {}", manifest.id);
    let _ = writeln!(out, "  version:  {}", manifest.version);
    let _ = writeln!(out, "  kind:     {}", manifest.kind.as_str());
    let _ = writeln!(out, "  declared: {}", format_caps(&manifest.capabilities));
    let _ = writeln!(out, "  files:    {}", files.len());
    let _ = writeln!(out, "  hash:     {package_hash}");
    match &signature {
        Some(sig) => {
            let _ = writeln!(out, "  signed:   {} (keyed HMAC-SHA256 over hash)", sig.signer);
        }
        None => {
            let _ = writeln!(out, "  signed:   no (install will require --allow-unverified)");
        }
    }
    Ok(out)
}

/// Walks `dir`, returning the package file map keyed by relative path. The
/// manifest source (`plugin.manifest`, else `plugin.toml`) is normalized to the
/// reserved `plugin.manifest` logical path.
fn collect_package_files(dir: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if !dir.is_dir() {
        return Err(format!("{} is not a directory", dir.display()));
    }
    let mut files = BTreeMap::new();
    walk(dir, dir, &mut files)?;

    let has_manifest = files.contains_key("plugin.manifest");
    if !has_manifest {
        if let Some(toml) = files.remove("plugin.toml") {
            files.insert("plugin.manifest".to_owned(), toml);
        }
    }
    if !files.contains_key("plugin.manifest") {
        return Err(format!("{} has no plugin.manifest or plugin.toml", dir.display()));
    }
    if files.contains_key("CHECKSUMS") || files.contains_key("SIGNATURE") {
        return Err("plugin directory must not contain reserved CHECKSUMS/SIGNATURE files".into());
    }
    Ok(files)
}

fn walk(root: &Path, dir: &Path, files: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let mut entries = fs::read_dir(dir)
        .map_err(|error| format!("failed to read {}: {error}", dir.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    entries.sort_by_key(std::fs::DirEntry::file_name);
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let file_type = entry.file_type().map_err(|error| error.to_string())?;
        if file_type.is_dir() {
            if EXCLUDED_DIRS.contains(&name.as_ref()) {
                continue;
            }
            walk(root, &path, files)?;
        } else if file_type.is_file() {
            if name.ends_with(".lsplugin") {
                continue;
            }
            let relative = path
                .strip_prefix(root)
                .map_err(|error| error.to_string())?
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            let bytes = fs::read(&path)
                .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
            files.insert(relative, bytes);
        }
    }
    Ok(())
}

fn parse_manifest_from(files: &BTreeMap<String, Vec<u8>>) -> Result<PluginManifest, String> {
    let bytes = files.get("plugin.manifest").ok_or("package is missing plugin.manifest")?;
    let text = std::str::from_utf8(bytes).map_err(|_| "plugin.manifest is not valid UTF-8")?;
    PluginManifest::parse(text).map_err(|error| error.to_string())
}

// ---------------------------------------------------------------------------
// install
// ---------------------------------------------------------------------------

fn install_command(arguments: &[String]) -> Result<String, String> {
    let mut package_path = None;
    let mut grant = None;
    let mut allow_unverified = false;
    let mut force = false;
    let mut dir = None;
    let mut trust = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--grant" => {
                grant = Some(value_of(arguments, index, "--grant")?);
                index += 2;
            }
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            "--trust" => {
                trust = Some(value_of(arguments, index, "--trust")?);
                index += 2;
            }
            "--allow-unverified" => {
                allow_unverified = true;
                index += 1;
            }
            "--force" => {
                force = true;
                index += 1;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => {
                if package_path.is_some() {
                    return Err("install takes a single PKG.lsplugin".to_owned());
                }
                package_path = Some(arguments[index].clone());
                index += 1;
            }
        }
    }
    let package_path = package_path.ok_or("usage: plugin install PKG.lsplugin [...]")?;
    let plugins_dir = plugins_dir(dir.as_deref())?;

    let bytes = fs::read(&package_path)
        .map_err(|error| format!("failed to read {package_path}: {error}"))?;
    let package = unpack(&bytes).map_err(|error| error.to_string())?;
    let manifest = package.manifest().clone();

    // Trust: verify the (optional) signature against the configured trust set.
    let store = load_trust_store(&plugins_dir, trust.as_deref())?;
    let trust_status = match package.signature() {
        Some(signature) => match store.verify(signature, package.package_hash()) {
            Some(signer) => TrustStatus::Trusted(signer),
            None => TrustStatus::Untrusted,
        },
        None => TrustStatus::Untrusted,
    };
    if !trust_status.is_trusted() && !allow_unverified {
        let reason = match package.signature() {
            None => "package is unsigned".to_owned(),
            Some(sig) => {
                format!("signature by {:?} did not verify against the trust set", sig.signer)
            }
        };
        return Err(format!(
            "refusing to install untrusted plugin ({reason}); pass --allow-unverified to install it as untrusted"
        ));
    }

    // Grants: default to none; must be a subset of the declared capabilities.
    let granted = parse_caps(grant.as_deref().unwrap_or(""))?;
    let installed = InstalledPlugin::new(
        manifest.clone(),
        granted.clone(),
        trust_status.clone(),
        package.package_hash().to_owned(),
    )
    .map_err(|error| error.to_string())?;

    // Non-destructive: refuse to replace an existing install of a different (or
    // same) version unless --force.
    let target = plugins_dir.join(&manifest.id);
    if let Some(existing) = read_install_record(&target)? {
        if !force {
            let note = if existing.version == manifest.version {
                format!("already installed at version {}", existing.version)
            } else {
                format!(
                    "installed version {} differs from package version {}",
                    existing.version, manifest.version
                )
            };
            return Err(format!("refusing to replace {} ({note}); pass --force", manifest.id));
        }
    }

    fs::create_dir_all(&target)
        .map_err(|error| format!("failed to create {}: {error}", target.display()))?;
    fs::write(target.join(PACKAGE_FILE), &bytes)
        .map_err(|error| format!("failed to write package: {error}"))?;
    fs::write(target.join(INSTALL_FILE), render_install_record(&installed))
        .map_err(|error| format!("failed to write install record: {error}"))?;

    let mut out = format!("installed {} {}\n", manifest.id, manifest.version);
    let _ = writeln!(out, "  into:     {}", target.display());
    let _ = writeln!(out, "  hash:     {}", installed.package_hash());
    let _ = writeln!(out, "  trust:    {}", format_trust(installed.trust()));
    let _ = writeln!(out, "  declared: {}", format_caps(installed.declared()));
    let _ = writeln!(out, "  granted:  {}", format_caps(installed.granted()));
    let limits = installed.limits();
    let _ = writeln!(
        out,
        "  limits:   cpu={}ms mem={}B out={}B reqs={}",
        limits.max_cpu_millis,
        limits.max_memory_bytes,
        limits.max_output_bytes,
        limits.max_requests
    );
    if !trust_status.is_trusted() {
        let _ = writeln!(out, "  note:     UNTRUSTED (installed with --allow-unverified)");
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

fn list_command(arguments: &[String]) -> Result<String, String> {
    let dir = single_dir_flag(arguments)?;
    let plugins_dir = plugins_dir(dir.as_deref())?;
    let records = installed_records(&plugins_dir)?;
    if records.is_empty() {
        return Ok(format!("no plugins installed in {}\n", plugins_dir.display()));
    }
    let mut out = format!("installed plugins in {}:\n", plugins_dir.display());
    for record in &records {
        let _ = writeln!(
            out,
            "  {}  {}  [{}]  granted={}",
            record.id,
            record.version,
            record.trust_label(),
            if record.granted.is_empty() { "-" } else { &record.granted }
        );
    }
    Ok(out)
}

// ---------------------------------------------------------------------------
// verify
// ---------------------------------------------------------------------------

fn verify_command(arguments: &[String]) -> Result<String, String> {
    let mut id = None;
    let mut dir = None;
    let mut trust = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            "--trust" => {
                trust = Some(value_of(arguments, index, "--trust")?);
                index += 2;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => {
                if id.is_some() {
                    return Err("verify takes a single ID".to_owned());
                }
                id = Some(arguments[index].clone());
                index += 1;
            }
        }
    }
    let id = id.ok_or("usage: plugin verify ID [--dir DIR] [--trust KEYSFILE]")?;
    let plugins_dir = plugins_dir(dir.as_deref())?;
    let target = plugins_dir.join(&id);
    let record = read_install_record(&target)?
        .ok_or_else(|| format!("{id} is not installed in {}", plugins_dir.display()))?;

    // Re-read and fully re-verify the stored package (checksums + container).
    let bytes = fs::read(target.join(PACKAGE_FILE))
        .map_err(|error| format!("failed to read stored package: {error}"))?;
    let package = unpack(&bytes).map_err(|error| format!("VERIFY FAILED for {id}: {error}"))?;

    // The recomputed package hash must match the one recorded at install time.
    if package.package_hash() != record.package_hash {
        return Err(format!(
            "VERIFY FAILED for {id}: package hash changed (recorded {}, got {})",
            short(&record.package_hash),
            short(package.package_hash())
        ));
    }

    // Re-verify the signature against the trust set (state may have changed).
    let store = load_trust_store(&plugins_dir, trust.as_deref())?;
    let trust_status = match package.signature() {
        Some(signature) => match store.verify(signature, package.package_hash()) {
            Some(signer) => TrustStatus::Trusted(signer),
            None => TrustStatus::Untrusted,
        },
        None => TrustStatus::Untrusted,
    };

    // Rebuild the installed view to expose the enforced capability decisions.
    let granted = parse_caps(&record.granted)?;
    let installed = InstalledPlugin::new(
        package.manifest().clone(),
        granted,
        trust_status.clone(),
        package.package_hash().to_owned(),
    )
    .map_err(|error| error.to_string())?;

    let mut out = format!("VERIFY OK for {id} {}\n", installed.manifest().version);
    let _ =
        writeln!(out, "  checksums: {} file(s) match CHECKSUMS manifest", package.files().len());
    let _ = writeln!(out, "  hash:      {} (matches install record)", installed.package_hash());
    match package.signature() {
        Some(sig) => {
            let _ = writeln!(
                out,
                "  signature: present, signer={:?} -> {}",
                sig.signer,
                if trust_status.is_trusted() {
                    "VERIFIED (trusted)"
                } else {
                    "NOT in trust set (untrusted)"
                }
            );
        }
        None => {
            let _ = writeln!(out, "  signature: none (untrusted)");
        }
    }
    let _ = writeln!(out, "  trust:     {}", format_trust(installed.trust()));
    let _ = writeln!(out, "  declared:  {}", format_caps(installed.declared()));
    let _ = writeln!(out, "  granted:   {}", format_caps(installed.granted()));
    // Make the enforcement point visible: decide each declared capability.
    let _ = writeln!(out, "  capability decisions (declared & granted => available):");
    for capability in installed.declared().iter() {
        let _ = writeln!(
            out,
            "    {:<16} {}",
            capability.as_str(),
            if installed.available(capability) { "available" } else { "DENIED (not granted)" }
        );
    }
    let limits = installed.limits();
    let _ = writeln!(
        out,
        "  limits:    cpu={}ms mem={}B out={}B reqs={}",
        limits.max_cpu_millis,
        limits.max_memory_bytes,
        limits.max_output_bytes,
        limits.max_requests
    );
    Ok(out)
}

// ---------------------------------------------------------------------------
// remove
// ---------------------------------------------------------------------------

fn remove_command(arguments: &[String]) -> Result<String, String> {
    let mut id = None;
    let mut dir = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => {
                if id.is_some() {
                    return Err("remove takes a single ID".to_owned());
                }
                id = Some(arguments[index].clone());
                index += 1;
            }
        }
    }
    let id = id.ok_or("usage: plugin remove ID [--dir DIR]")?;
    let plugins_dir = plugins_dir(dir.as_deref())?;
    let target = plugins_dir.join(&id);
    if read_install_record(&target)?.is_none() {
        return Err(format!("{id} is not installed in {}", plugins_dir.display()));
    }
    fs::remove_dir_all(&target)
        .map_err(|error| format!("failed to remove {}: {error}", target.display()))?;
    Ok(format!("removed {id} from {}\n", plugins_dir.display()))
}

// ---------------------------------------------------------------------------
// registry
// ---------------------------------------------------------------------------

fn registry_command(arguments: &[String]) -> Result<String, String> {
    let Some(subcommand) = arguments.first().map(String::as_str) else {
        return Err("usage: plugin registry <add|list> ...".to_owned());
    };
    match subcommand {
        "add" => registry_add(&arguments[1..]),
        "list" => registry_list(&arguments[1..]),
        _ => Err("usage: plugin registry <add|list> ...".to_owned()),
    }
}

fn registry_add(arguments: &[String]) -> Result<String, String> {
    let mut package_path = None;
    let mut dir = None;
    let mut registry = None;
    let mut location = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            "--registry" => {
                registry = Some(value_of(arguments, index, "--registry")?);
                index += 2;
            }
            "--location" => {
                location = Some(value_of(arguments, index, "--location")?);
                index += 2;
            }
            flag if flag.starts_with("--") => return Err(format!("unknown flag {flag}")),
            _ => {
                if package_path.is_some() {
                    return Err("registry add takes a single PKG.lsplugin".to_owned());
                }
                package_path = Some(arguments[index].clone());
                index += 1;
            }
        }
    }
    let package_path = package_path.ok_or("usage: plugin registry add PKG.lsplugin [...]")?;
    let plugins_dir = plugins_dir(dir.as_deref())?;
    let registry_path =
        registry.map(PathBuf::from).unwrap_or_else(|| plugins_dir.join(REGISTRY_FILE));

    let bytes = fs::read(&package_path)
        .map_err(|error| format!("failed to read {package_path}: {error}"))?;
    let package = unpack(&bytes).map_err(|error| error.to_string())?;
    let manifest = package.manifest();
    let signer = package.signature().map(|s| s.signer.clone()).unwrap_or_else(|| "-".to_owned());
    let location = location.unwrap_or(package_path);

    let mut entries = read_registry(&registry_path)?;
    let key = (manifest.id.clone(), manifest.version.clone());
    entries.insert(
        key,
        RegistryEntry {
            package_hash: package.package_hash().to_owned(),
            location: sanitize(&location),
            signer: sanitize(&signer),
        },
    );
    write_registry(&registry_path, &entries)?;

    Ok(format!(
        "registered {} {} [{}] -> {}\n  index: {}\n",
        manifest.id,
        manifest.version,
        short(package.package_hash()),
        location_display(&entries, &manifest.id, &manifest.version),
        registry_path.display()
    ))
}

fn registry_list(arguments: &[String]) -> Result<String, String> {
    let mut dir = None;
    let mut registry = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            "--registry" => {
                registry = Some(value_of(arguments, index, "--registry")?);
                index += 2;
            }
            flag => return Err(format!("unknown flag {flag}")),
        }
    }
    let plugins_dir = plugins_dir(dir.as_deref())?;
    let registry_path =
        registry.map(PathBuf::from).unwrap_or_else(|| plugins_dir.join(REGISTRY_FILE));
    let entries = read_registry(&registry_path)?;
    if entries.is_empty() {
        return Ok(format!("registry {} is empty\n", registry_path.display()));
    }
    let mut out = format!("registry {}:\n", registry_path.display());
    for ((id, version), entry) in &entries {
        let _ = writeln!(
            out,
            "  {id}  {version}  [{}]  signer={}  {}",
            short(&entry.package_hash),
            entry.signer,
            entry.location
        );
    }
    Ok(out)
}

struct RegistryEntry {
    package_hash: String,
    location: String,
    signer: String,
}

fn read_registry(path: &Path) -> Result<BTreeMap<(String, String), RegistryEntry>, String> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let mut entries = BTreeMap::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with("id\tversion") || line.starts_with('#') {
            continue;
        }
        let fields: Vec<&str> = line.splitn(5, '\t').collect();
        if fields.len() != 5 {
            return Err(format!("malformed registry line: {line}"));
        }
        entries.insert(
            (fields[0].to_owned(), fields[1].to_owned()),
            RegistryEntry {
                package_hash: fields[2].to_owned(),
                location: fields[4].to_owned(),
                signer: fields[3].to_owned(),
            },
        );
    }
    Ok(entries)
}

fn write_registry(
    path: &Path,
    entries: &BTreeMap<(String, String), RegistryEntry>,
) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    let mut out = String::from(REGISTRY_HEADER);
    out.push('\n');
    for ((id, version), entry) in entries {
        let _ = writeln!(
            out,
            "{id}\t{version}\t{}\t{}\t{}",
            entry.package_hash, entry.signer, entry.location
        );
    }
    fs::write(path, out).map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn location_display(
    entries: &BTreeMap<(String, String), RegistryEntry>,
    id: &str,
    version: &str,
) -> String {
    entries
        .get(&(id.to_owned(), version.to_owned()))
        .map(|entry| entry.location.clone())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Install record (install.toml) — deterministic key = value text
// ---------------------------------------------------------------------------

struct InstallRecord {
    id: String,
    version: String,
    package_hash: String,
    trust: String,
    granted: String,
}

impl InstallRecord {
    fn trust_label(&self) -> &str {
        if self.trust == "untrusted" { "untrusted" } else { "trusted" }
    }
}

fn render_install_record(installed: &InstalledPlugin) -> String {
    let manifest = installed.manifest();
    let limits = installed.limits();
    let (trust, signer) = match installed.trust() {
        TrustStatus::Trusted(signer) => ("trusted".to_owned(), signer.clone()),
        TrustStatus::Untrusted => ("untrusted".to_owned(), "-".to_owned()),
    };
    let mut out = String::new();
    let _ = writeln!(out, "id = {}", manifest.id);
    let _ = writeln!(out, "version = {}", manifest.version);
    let _ = writeln!(out, "kind = {}", manifest.kind.as_str());
    let _ = writeln!(out, "package_hash = {}", installed.package_hash());
    let _ = writeln!(out, "trust = {trust}");
    let _ = writeln!(out, "signer = {signer}");
    let _ = writeln!(out, "declared = {}", caps_field(installed.declared()));
    let _ = writeln!(out, "granted = {}", caps_field(installed.granted()));
    let _ = writeln!(out, "max_cpu_millis = {}", limits.max_cpu_millis);
    let _ = writeln!(out, "max_memory_bytes = {}", limits.max_memory_bytes);
    let _ = writeln!(out, "max_output_bytes = {}", limits.max_output_bytes);
    let _ = writeln!(out, "max_requests = {}", limits.max_requests);
    out
}

fn read_install_record(target: &Path) -> Result<Option<InstallRecord>, String> {
    let path = target.join(INSTALL_FILE);
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    let mut fields: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, value) =
            line.split_once('=').ok_or_else(|| format!("malformed install record line: {line}"))?;
        fields.insert(key.trim().to_owned(), value.trim().to_owned());
    }
    let get = |key: &str| -> Result<String, String> {
        fields.get(key).cloned().ok_or_else(|| format!("install record missing {key}"))
    };
    Ok(Some(InstallRecord {
        id: get("id")?,
        version: get("version")?,
        package_hash: get("package_hash")?,
        trust: get("trust")?,
        granted: fields.get("granted").cloned().unwrap_or_else(|| "-".to_owned()),
    }))
}

fn installed_records(plugins_dir: &Path) -> Result<Vec<InstallRecord>, String> {
    let read_dir = match fs::read_dir(plugins_dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", plugins_dir.display())),
    };
    let mut directories =
        read_dir.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())?;
    directories.sort_by_key(std::fs::DirEntry::file_name);
    let mut records = Vec::new();
    for entry in directories {
        if !entry.file_type().map_err(|error| error.to_string())?.is_dir() {
            continue;
        }
        if let Some(record) = read_install_record(&entry.path())? {
            records.push(record);
        }
    }
    Ok(records)
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

fn plugins_dir(dir_override: Option<&str>) -> Result<PathBuf, String> {
    match dir_override {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => {
            let home = std::env::var("HOME").map_err(|_| {
                "HOME is not set; pass --dir to choose a plugin directory".to_owned()
            })?;
            Ok(PathBuf::from(home).join(".lawsynth").join("plugins"))
        }
    }
}

fn load_trust_store(plugins_dir: &Path, explicit: Option<&str>) -> Result<TrustStore, String> {
    let path = match explicit {
        Some(path) => PathBuf::from(path),
        None => plugins_dir.join(DEFAULT_TRUST_FILE),
    };
    let text = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if explicit.is_some() {
                return Err(format!("trust set not found: {}", path.display()));
            }
            return Ok(TrustStore::default());
        }
        Err(error) => return Err(format!("failed to read {}: {error}", path.display())),
    };
    TrustStore::parse(&text).map_err(|error| error.to_string())
}

/// Reads a signing keyfile: `signer = NAME` and `secret = <hex>` lines.
fn read_signing_key(path: &str) -> Result<(String, Vec<u8>), String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("failed to read signing key {path}: {error}"))?;
    let mut signer = None;
    let mut secret = None;
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) =
            line.split_once('=').ok_or_else(|| format!("malformed signing key line: {line}"))?;
        match key.trim() {
            "signer" => signer = Some(value.trim().to_owned()),
            "secret" => secret = Some(value.trim().to_owned()),
            other => return Err(format!("unknown signing key field {other:?}")),
        }
    }
    let signer = signer.ok_or("signing key file missing `signer =`")?;
    let secret_hex = secret.ok_or("signing key file missing `secret =`")?;
    let secret = decode_hex(&secret_hex).ok_or("`secret` must be lowercase hex bytes")?;
    if signer.is_empty() {
        return Err("`signer` must not be empty".to_owned());
    }
    Ok((signer, secret))
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

fn parse_caps(text: &str) -> Result<CapabilitySet, String> {
    let mut caps = Vec::new();
    for token in text.split(',') {
        let token = token.trim();
        if token.is_empty() || token == "-" || token == "none" {
            continue;
        }
        caps.push(token.parse::<Capability>().map_err(|error| error.to_string())?);
    }
    Ok(CapabilitySet::new(caps))
}

fn format_caps(caps: &CapabilitySet) -> String {
    if caps.is_empty() {
        "none".to_owned()
    } else {
        caps.iter().map(Capability::as_str).collect::<Vec<_>>().join(", ")
    }
}

fn caps_field(caps: &CapabilitySet) -> String {
    if caps.is_empty() {
        "-".to_owned()
    } else {
        caps.iter().map(Capability::as_str).collect::<Vec<_>>().join(",")
    }
}

fn format_trust(status: &TrustStatus) -> String {
    match status {
        TrustStatus::Trusted(signer) => format!("trusted (signer: {signer})"),
        TrustStatus::Untrusted => "untrusted".to_owned(),
    }
}

fn value_of(arguments: &[String], index: usize, flag: &str) -> Result<String, String> {
    arguments.get(index + 1).cloned().ok_or_else(|| format!("missing value for {flag}"))
}

fn single_dir_flag(arguments: &[String]) -> Result<Option<String>, String> {
    let mut dir = None;
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                dir = Some(value_of(arguments, index, "--dir")?);
                index += 2;
            }
            flag => return Err(format!("unknown flag {flag}")),
        }
    }
    Ok(dir)
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn short(hash: &str) -> String {
    if hash.len() > 12 { format!("{}…", &hash[..12]) } else { hash.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_caps_handles_none_and_lists() {
        assert!(parse_caps("").unwrap().is_empty());
        assert!(parse_caps("none").unwrap().is_empty());
        let caps = parse_caps("world.validate, dataset.read").unwrap();
        assert!(caps.contains(Capability::WorldValidate));
        assert!(caps.contains(Capability::ReadDataset));
        assert!(parse_caps("bogus.cap").is_err());
    }

    #[test]
    fn caps_field_round_trips() {
        let caps = CapabilitySet::new([Capability::WorldValidate, Capability::WriteArtifact]);
        let field = caps_field(&caps);
        assert_eq!(parse_caps(&field).unwrap(), caps);
        assert_eq!(caps_field(&CapabilitySet::default()), "-");
    }

    #[test]
    fn decode_hex_rejects_odd_and_nonhex() {
        assert_eq!(decode_hex("dead"), Some(vec![0xde, 0xad]));
        assert!(decode_hex("abc").is_none());
        assert!(decode_hex("xy").is_none());
    }
}
