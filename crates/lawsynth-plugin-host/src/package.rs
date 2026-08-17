//! Self-describing `.lsplugin` package format: pack, unpack, and verify.
//!
//! A `.lsplugin` package is a std-only container that mirrors the CLI's
//! `.lsworkspace` archive: a UTF-8 magic header, a path-sorted per-entry
//! manifest with SHA-256 integrity, a payload marker, then the concatenated raw
//! payloads. It carries a plugin's manifest, its artifact file(s), a checksum
//! manifest (SHA-256 per file), and an optional keyed signature.
//!
//! ```text
//! LSPLUGIN\tv1\n
//! entry\t<byte_len>\t<sha256_hex>\t<logical_path>\n   (one per entry, path-sorted)
//! --payload--\n
//! <payload bytes, concatenated in the same order as the manifest lines>
//! ```
//!
//! Reserved logical paths:
//! - `CHECKSUMS`       the checksum manifest (SHA-256 per packaged file).
//! - `plugin.manifest` the plugin manifest (existing `key = value` grammar).
//! - `SIGNATURE`       optional `signer\ttag` keyed-HMAC authenticity tag.
//!
//! The **package hash** is the SHA-256 of the canonical `CHECKSUMS` bytes — it
//! is independent of the (optional) signature, so signing never changes the
//! identity of a package. Every packaged file is hashed twice over: once in the
//! container manifest (transport integrity) and once in `CHECKSUMS` (the
//! spec-required per-file checksum manifest that defines the package hash).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use lawsynth_bundle::sha256_hex;
use lawsynth_plugin_api::PluginManifest;

use crate::HostError;

const MAGIC: &str = "LSPLUGIN\tv1";
const PAYLOAD_MARKER: &[u8] = b"--payload--\n";
const CHECKSUMS_HEADER: &str = "lsplugin-checksums\tv1";

/// Reserved logical path of the checksum manifest inside a package.
pub const CHECKSUMS_PATH: &str = "CHECKSUMS";
/// Reserved logical path of the signature tag inside a package.
pub const SIGNATURE_PATH: &str = "SIGNATURE";
/// Reserved logical path of the plugin manifest inside a package.
pub const MANIFEST_PATH: &str = "plugin.manifest";

/// A keyed-HMAC authenticity tag recorded inside a package.
///
/// Honesty: this is a **symmetric** MAC over the package hash, not a public-key
/// signature. Anyone holding the shared secret can both create and verify it, so
/// it proves *integrity + authenticity relative to a shared key*, never
/// non-repudiable authorship.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageSignature {
    pub signer: String,
    pub tag: String,
}

/// A parsed, integrity-verified plugin package.
#[derive(Clone, Debug)]
pub struct PluginPackage {
    manifest: PluginManifest,
    files: BTreeMap<String, Vec<u8>>,
    package_hash: String,
    signature: Option<PackageSignature>,
}

impl PluginPackage {
    /// The validated plugin manifest.
    pub fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }
    /// The content hash of the checksum manifest (the mirror-stable identity).
    pub fn package_hash(&self) -> &str {
        &self.package_hash
    }
    /// The optional keyed-HMAC signature.
    pub fn signature(&self) -> Option<&PackageSignature> {
        self.signature.as_ref()
    }
    /// The packaged files (includes `plugin.manifest`; excludes `CHECKSUMS`
    /// and `SIGNATURE`).
    pub fn files(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.files
    }
}

/// Serializes the canonical checksum manifest for `files` (path-sorted).
///
/// `files` MUST exclude the reserved `CHECKSUMS`/`SIGNATURE` entries.
pub fn build_checksums(files: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut out = String::from(CHECKSUMS_HEADER);
    out.push('\n');
    for (path, bytes) in files {
        let _ = writeln!(out, "{}  {}", sha256_hex(bytes), path);
    }
    out.into_bytes()
}

/// The package hash: SHA-256 of the canonical checksum manifest bytes.
pub fn package_hash_of(checksums: &[u8]) -> String {
    sha256_hex(checksums)
}

/// Packs a plugin's `files` (which MUST contain `plugin.manifest`) into a
/// `.lsplugin` container, optionally embedding a signature over the package
/// hash. Returns the serialized bytes and the package hash.
pub fn pack(
    files: &BTreeMap<String, Vec<u8>>,
    signature: Option<&PackageSignature>,
) -> Result<(Vec<u8>, String), HostError> {
    let manifest_bytes = files
        .get(MANIFEST_PATH)
        .ok_or_else(|| HostError::Package(format!("package is missing {MANIFEST_PATH}")))?;
    let manifest_text = std::str::from_utf8(manifest_bytes)
        .map_err(|_| HostError::Package("plugin.manifest is not valid UTF-8".into()))?;
    // Reject an invalid or unparsable manifest up front.
    PluginManifest::parse(manifest_text)?;
    if files.contains_key(CHECKSUMS_PATH) || files.contains_key(SIGNATURE_PATH) {
        return Err(HostError::Package(
            "packaged files must not use the reserved CHECKSUMS/SIGNATURE paths".into(),
        ));
    }

    let checksums = build_checksums(files);
    let package_hash = package_hash_of(&checksums);

    let mut entries: BTreeMap<String, Vec<u8>> = files.clone();
    entries.insert(CHECKSUMS_PATH.to_owned(), checksums);
    if let Some(sig) = signature {
        entries.insert(SIGNATURE_PATH.to_owned(), encode_signature(sig));
    }
    Ok((write_container(&entries), package_hash))
}

/// Parses and fully verifies a `.lsplugin` container: per-entry transport
/// integrity, the checksum manifest against every packaged file, and manifest
/// validity. Returns the parsed package (signature parsed but not yet trusted).
pub fn unpack(bytes: &[u8]) -> Result<PluginPackage, HostError> {
    let mut entries = read_container(bytes)?;

    let checksums = entries
        .remove(CHECKSUMS_PATH)
        .ok_or_else(|| HostError::Package("package is missing CHECKSUMS".into()))?;
    let signature = entries.remove(SIGNATURE_PATH).map(|raw| decode_signature(&raw)).transpose()?;

    // The remaining entries are the packaged files; the checksum manifest must
    // describe exactly this set, byte for byte.
    let expected = build_checksums(&entries);
    if expected != checksums {
        return Err(HostError::Package(
            "checksum manifest does not match packaged files (tampered or corrupt)".into(),
        ));
    }
    let package_hash = package_hash_of(&checksums);

    let manifest_bytes = entries
        .get(MANIFEST_PATH)
        .ok_or_else(|| HostError::Package(format!("package is missing {MANIFEST_PATH}")))?;
    let manifest_text = std::str::from_utf8(manifest_bytes)
        .map_err(|_| HostError::Package("plugin.manifest is not valid UTF-8".into()))?;
    let manifest = PluginManifest::parse(manifest_text)?;

    Ok(PluginPackage { manifest, files: entries, package_hash, signature })
}

fn encode_signature(signature: &PackageSignature) -> Vec<u8> {
    format!("{}\t{}\n", sanitize(&signature.signer), signature.tag).into_bytes()
}

fn decode_signature(raw: &[u8]) -> Result<PackageSignature, HostError> {
    let text = std::str::from_utf8(raw)
        .map_err(|_| HostError::Package("SIGNATURE is not valid UTF-8".into()))?;
    let line = text.lines().next().unwrap_or("");
    let (signer, tag) = line
        .split_once('\t')
        .ok_or_else(|| HostError::Package("SIGNATURE must be `signer\\ttag`".into()))?;
    if signer.is_empty() || tag.is_empty() {
        return Err(HostError::Package("SIGNATURE has an empty field".into()));
    }
    Ok(PackageSignature { signer: signer.to_owned(), tag: tag.to_owned() })
}

/// Serializes container `entries` into the `.lsplugin` format.
fn write_container(entries: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut header = String::from(MAGIC);
    header.push('\n');
    for (path, content) in entries {
        let _ = writeln!(header, "entry\t{}\t{}\t{path}", content.len(), sha256_hex(content));
    }
    let mut out = header.into_bytes();
    out.extend_from_slice(PAYLOAD_MARKER);
    for content in entries.values() {
        out.extend_from_slice(content);
    }
    out
}

/// Parses a `.lsplugin` container, verifying per-entry SHA-256 integrity.
fn read_container(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, HostError> {
    let marker = find_subslice(bytes, PAYLOAD_MARKER).ok_or_else(|| {
        HostError::Package("not a LawSynth plugin package (missing marker)".into())
    })?;
    let header = std::str::from_utf8(&bytes[..marker])
        .map_err(|_| HostError::Package("package header is not valid UTF-8".into()))?;
    let mut lines = header.lines();
    if lines.next() != Some(MAGIC) {
        return Err(HostError::Package("unsupported package format (expected LSPLUGIN v1)".into()));
    }

    struct Line {
        len: usize,
        hash: String,
        path: String,
    }
    let mut manifest = Vec::new();
    for line in lines {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.splitn(4, '\t').collect();
        if fields.len() != 4 || fields[0] != "entry" {
            return Err(HostError::Package(format!("malformed package manifest line: {line}")));
        }
        let len = fields[1]
            .parse::<usize>()
            .map_err(|_| HostError::Package(format!("invalid entry length: {}", fields[1])))?;
        manifest.push(Line { len, hash: fields[2].to_owned(), path: fields[3].to_owned() });
    }

    let mut cursor = marker + PAYLOAD_MARKER.len();
    let mut entries = BTreeMap::new();
    for entry in manifest {
        let end = cursor
            .checked_add(entry.len)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| HostError::Package(format!("package is truncated at {}", entry.path)))?;
        let content = bytes[cursor..end].to_vec();
        let actual = sha256_hex(&content);
        if actual != entry.hash {
            return Err(HostError::Package(format!(
                "integrity check failed for {}: expected {}, got {}",
                entry.path,
                short(&entry.hash),
                short(&actual)
            )));
        }
        if entries.insert(entry.path.clone(), content).is_some() {
            return Err(HostError::Package(format!("duplicate package entry: {}", entry.path)));
        }
        cursor = end;
    }
    Ok(entries)
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len())
        .find(|&start| &haystack[start..start + needle.len()] == needle)
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

    fn sample_manifest() -> Vec<u8> {
        b"id = demo\nversion = 1.0.0\nkind = wasi\nentrypoint = demo.wasm\ncapabilities = world.validate\n".to_vec()
    }

    fn sample_files() -> BTreeMap<String, Vec<u8>> {
        let mut files = BTreeMap::new();
        files.insert(MANIFEST_PATH.to_owned(), sample_manifest());
        files.insert("demo.wasm".to_owned(), vec![0, 1, 2, 3, 255]);
        files.insert("README.md".to_owned(), b"# demo".to_vec());
        files
    }

    #[test]
    fn pack_unpack_round_trips_with_stable_hash() {
        let files = sample_files();
        let (bytes, hash) = pack(&files, None).unwrap();
        let package = unpack(&bytes).unwrap();
        assert_eq!(package.package_hash(), hash);
        assert_eq!(package.files(), &files);
        assert!(package.signature().is_none());
        assert_eq!(package.manifest().id, "demo");
    }

    #[test]
    fn signing_does_not_change_the_package_hash() {
        let files = sample_files();
        let (unsigned, hash_unsigned) = pack(&files, None).unwrap();
        let sig = PackageSignature { signer: "acme".into(), tag: "deadbeef".into() };
        let (signed, hash_signed) = pack(&files, Some(&sig)).unwrap();
        assert_eq!(hash_unsigned, hash_signed);
        assert_ne!(unsigned, signed);
        assert_eq!(unpack(&signed).unwrap().signature(), Some(&sig));
    }

    #[test]
    fn tampering_a_payload_byte_is_detected() {
        let files = sample_files();
        let (mut bytes, _) = pack(&files, None).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        assert!(unpack(&bytes).is_err());
    }

    #[test]
    fn missing_manifest_is_rejected() {
        let mut files = BTreeMap::new();
        files.insert("demo.wasm".to_owned(), vec![1, 2, 3]);
        assert!(pack(&files, None).is_err());
    }
}
