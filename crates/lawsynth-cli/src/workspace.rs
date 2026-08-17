//! `lawsynth workspace export|import` — make a whole workspace portable.
//!
//! A workspace lives under `~/.lawsynth/` (override with `--dir`): a library
//! index (`library.tsv`), the `.lsworld` bundles it points at, and a runs
//! registry (`runs/*.run`). `export` bundles all three — the actual bundle
//! bytes, the index, and every run record — into ONE self-describing
//! `.lsworkspace` archive with per-entry SHA-256 integrity. `import` unpacks it
//! into a target workspace, non-destructively: it refuses to clobber existing
//! library names unless `--force`, and reports exactly what was imported.
//!
//! ## Container format (`.lsworkspace`, v1)
//!
//! A UTF-8 header, a sentinel line, then the concatenated raw payloads:
//!
//! ```text
//! LSWORKSPACE\tv1\n
//! entry\t<byte_len>\t<sha256_hex>\t<logical_path>\n   (one per entry, path-sorted)
//! --payload--\n
//! <payload bytes, concatenated in the same order as the manifest lines>
//! ```
//!
//! Logical paths: `library.tsv`, `worlds.tsv` (a `name\tcontainer_path\tsha256\
//! toriginal_path` map), `worlds/<NNNN>.lsworld`, and `runs/<id>.run`. Everything
//! is sorted, so the archive is byte-deterministic — no wall clock is read.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::{read_world, sha256_hex};

use crate::library::HEADER as LIBRARY_HEADER;

const MAGIC: &str = "LSWORKSPACE\tv1";
const PAYLOAD_MARKER: &[u8] = b"--payload--\n";

/// Help text for `lawsynth workspace`.
pub fn help() -> String {
    "lawsynth workspace <export|import> ...\n\n\
  workspace export ARCHIVE.lsworkspace [--dir DIR]\n\
      Bundle the library index, its .lsworld bundles, and the runs registry from\n\
      DIR (default ~/.lawsynth) into one self-describing, SHA-256-checked archive.\n\
  workspace import ARCHIVE.lsworkspace [--dir DIR] [--force]\n\
      Unpack an archive into DIR (default ~/.lawsynth). Non-destructive: existing\n\
      library names are kept unless --force. Reports what was imported.\n\n\
The archive is portable and deterministic: share it with a colleague or move it\n\
between machines and the worlds + provenance survive byte-for-byte."
        .to_owned()
}

/// Runs the `workspace` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(subcommand) = arguments.first().map(String::as_str) else {
        return Err(help());
    };
    if subcommand == "--help" || subcommand == "-h" {
        return Ok(help());
    }
    let (dir_override, force, rest) = extract_flags(&arguments[1..])?;
    match subcommand {
        "export" => export(rest.first(), dir_override.as_deref()),
        "import" => import(rest.first(), dir_override.as_deref(), force),
        _ => Err(help()),
    }
}

/// A world staged for the archive: its library name, bundle bytes, and hash.
struct StagedWorld {
    name: String,
    container_path: String,
    original_path: String,
    hash: String,
    bytes: Vec<u8>,
}

fn export(archive: Option<&String>, dir_override: Option<&str>) -> Result<String, String> {
    let Some(archive) = archive else {
        return Err("usage: workspace export ARCHIVE.lsworkspace [--dir DIR]".to_owned());
    };
    let dir = workspace_dir(dir_override)?;
    let library_path = dir.join("library.tsv");
    let library_bytes = match fs::read(&library_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(format!(
                "no library index at {} — register worlds with `lawsynth library add` first",
                library_path.display()
            ));
        }
        Err(error) => return Err(format!("failed to read {}: {error}", library_path.display())),
    };

    // Resolve every library entry to its bundle bytes, sorted by name so the
    // container path assignment (worlds/NNNN.lsworld) is deterministic.
    let mut records = library_records(&library_bytes);
    records.sort_by(|left, right| left.0.cmp(&right.0));
    let mut staged = Vec::new();
    for (index, (name, path)) in records.iter().enumerate() {
        let bytes = fs::read(path)
            .map_err(|error| format!("failed to read bundle '{path}' for '{name}': {error}"))?;
        staged.push(StagedWorld {
            name: name.clone(),
            container_path: format!("worlds/{index:04}.lsworld"),
            original_path: path.clone(),
            hash: sha256_hex(&bytes),
            bytes,
        });
    }

    // Assemble the container entries (BTreeMap keeps them path-sorted).
    let mut entries: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    entries.insert("library.tsv".to_owned(), library_bytes);
    entries.insert("worlds.tsv".to_owned(), worlds_manifest(&staged).into_bytes());
    for world in &staged {
        entries.insert(world.container_path.clone(), world.bytes.clone());
    }
    let runs = read_runs(&dir.join("runs"))?;
    for (name, bytes) in &runs {
        entries.insert(format!("runs/{name}"), bytes.clone());
    }

    let container = write_container(&entries);
    fs::write(archive, &container)
        .map_err(|error| format!("failed to write {archive}: {error}"))?;

    let mut out = format!("exported workspace -> {archive} ({} bytes)\n", container.len());
    let _ = writeln!(out, "  from:   {}", dir.display());
    let _ = writeln!(out, "  worlds: {}", staged.len());
    let _ = writeln!(out, "  runs:   {}", runs.len());
    for world in &staged {
        let _ = writeln!(out, "  + {}  [{}]", world.name, short_hash(&world.hash));
    }
    Ok(out)
}

fn import(
    archive: Option<&String>,
    dir_override: Option<&str>,
    force: bool,
) -> Result<String, String> {
    let Some(archive) = archive else {
        return Err("usage: workspace import ARCHIVE.lsworkspace [--dir DIR] [--force]".to_owned());
    };
    let bytes = fs::read(archive).map_err(|error| format!("failed to read {archive}: {error}"))?;
    let entries = read_container(&bytes)?;

    let dir = workspace_dir(dir_override)?;
    let worlds_dir = dir.join("worlds");
    fs::create_dir_all(&worlds_dir)
        .map_err(|error| format!("failed to create {}: {error}", worlds_dir.display()))?;

    // Existing target names must not be clobbered unless --force.
    let library_path = dir.join("library.tsv");
    let existing_bytes = fs::read(&library_path).unwrap_or_default();
    let existing: Vec<(String, Vec<String>)> = library_lines(&existing_bytes);
    let existing_names: Vec<String> = existing.iter().map(|(name, _)| name.clone()).collect();

    // The container's world manifest maps names -> container payload + hash.
    let manifest = entries
        .get("worlds.tsv")
        .ok_or("archive is missing worlds.tsv; not a LawSynth workspace archive")?;
    let manifest = parse_worlds_manifest(manifest)?;
    let container_library = entries
        .get("library.tsv")
        .ok_or("archive is missing library.tsv; not a LawSynth workspace archive")?;
    let container_entries = library_lines(container_library);

    let mut imported = Vec::new();
    let mut skipped = Vec::new();
    let mut new_lines: Vec<(String, Vec<String>)> = existing.clone();
    for (name, fields) in container_entries {
        let Some(world) = manifest.iter().find(|entry| entry.name == name) else {
            // A library entry without a bundle in the archive: skip defensively.
            skipped.push(format!("{name} (no bundle in archive)"));
            continue;
        };
        if existing_names.contains(&name) && !force {
            skipped.push(format!("{name} (exists; use --force)"));
            continue;
        }
        let payload = entries
            .get(&world.container_path)
            .ok_or_else(|| format!("archive is missing payload {}", world.container_path))?;
        // Integrity: the payload must match the hash recorded at export time.
        let actual = sha256_hex(payload);
        if actual != world.hash {
            return Err(format!(
                "integrity check failed for '{name}': expected {}, got {}",
                short_hash(&world.hash),
                short_hash(&actual)
            ));
        }
        let target_bundle = worlds_dir.join(format!("{}.lsworld", safe_filename(&name)));
        fs::write(&target_bundle, payload)
            .map_err(|error| format!("failed to write {}: {error}", target_bundle.display()))?;
        // Confirm the bundle parses as a world before advertising it.
        read_world(&target_bundle)
            .map_err(|error| format!("imported bundle for '{name}' failed to parse: {error}"))?;

        // Rewrite the entry's path column to the freshly written bundle.
        let mut fields = fields;
        if fields.len() < 2 {
            fields.resize(2, String::new());
        }
        fields[1] = target_bundle.to_string_lossy().into_owned();
        new_lines.retain(|(existing_name, _)| existing_name != &name);
        new_lines.push((name.clone(), fields));
        imported.push((name, world.hash.clone()));
    }

    // Merge + persist the library index (header + name-sorted entries).
    new_lines.sort_by(|left, right| left.0.cmp(&right.0));
    let mut library_out = String::from(LIBRARY_HEADER);
    library_out.push('\n');
    for (_, fields) in &new_lines {
        library_out.push_str(&fields.join("\t"));
        library_out.push('\n');
    }
    fs::write(&library_path, library_out)
        .map_err(|error| format!("failed to write {}: {error}", library_path.display()))?;

    // Runs are content-addressed and idempotent — copy them in verbatim.
    let runs_dir = dir.join("runs");
    let mut runs_imported = 0;
    for (path, payload) in &entries {
        if let Some(run_name) = path.strip_prefix("runs/") {
            fs::create_dir_all(&runs_dir)
                .map_err(|error| format!("failed to create {}: {error}", runs_dir.display()))?;
            let target = runs_dir.join(run_name);
            fs::write(&target, payload)
                .map_err(|error| format!("failed to write {}: {error}", target.display()))?;
            runs_imported += 1;
        }
    }

    let mut out = format!("imported workspace <- {archive}\n");
    let _ = writeln!(out, "  into:     {}", dir.display());
    let _ = writeln!(out, "  worlds:   {} imported, {} skipped", imported.len(), skipped.len());
    let _ = writeln!(out, "  runs:     {runs_imported} imported");
    for (name, hash) in &imported {
        let _ = writeln!(out, "  + {name}  [{}]", short_hash(hash));
    }
    for note in &skipped {
        let _ = writeln!(out, "  - {note}");
    }
    Ok(out)
}

/// Serializes the workspace world manifest (`name\tcontainer\tsha256\toriginal`).
fn worlds_manifest(staged: &[StagedWorld]) -> String {
    let mut out = String::from("name\tcontainer_path\tsha256\toriginal_path\n");
    for world in staged {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}",
            sanitize(&world.name),
            world.container_path,
            world.hash,
            sanitize(&world.original_path)
        );
    }
    out
}

/// A parsed workspace world manifest row.
struct WorldEntry {
    name: String,
    container_path: String,
    hash: String,
}

fn parse_worlds_manifest(bytes: &[u8]) -> Result<Vec<WorldEntry>, String> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| "worlds.tsv is not valid UTF-8".to_owned())?;
    let mut entries = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with("name\tcontainer_path") {
            continue;
        }
        let fields: Vec<&str> = line.splitn(4, '\t').collect();
        if fields.len() < 3 {
            continue;
        }
        entries.push(WorldEntry {
            name: fields[0].to_owned(),
            container_path: fields[1].to_owned(),
            hash: fields[2].to_owned(),
        });
    }
    Ok(entries)
}

/// Extracts `(name, bundle_path)` pairs from raw `library.tsv` bytes.
fn library_records(bytes: &[u8]) -> Vec<(String, String)> {
    library_lines(bytes)
        .into_iter()
        .filter_map(|(name, fields)| fields.get(1).map(|path| (name, path.clone())))
        .collect()
}

/// Parses `library.tsv` data lines into `(name, all_fields)`, skipping headers.
fn library_lines(bytes: &[u8]) -> Vec<(String, Vec<String>)> {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with("name\tpath\t") {
            continue;
        }
        let fields: Vec<String> = line.split('\t').map(str::to_owned).collect();
        if fields.len() < 2 {
            continue;
        }
        lines.push((fields[0].clone(), fields));
    }
    lines
}

/// Reads every `*.run` file in `runs_dir`, returning `(filename, bytes)` sorted.
fn read_runs(runs_dir: &Path) -> Result<Vec<(String, Vec<u8>)>, String> {
    let read_dir = match fs::read_dir(runs_dir) {
        Ok(read_dir) => read_dir,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", runs_dir.display())),
    };
    let mut runs = Vec::new();
    for entry in read_dir {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("run") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let bytes = fs::read(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        runs.push((name.to_owned(), bytes));
    }
    runs.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(runs)
}

/// Serializes container `entries` into the `.lsworkspace` v1 format.
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

/// Parses a `.lsworkspace` container, verifying per-entry SHA-256 integrity.
fn read_container(bytes: &[u8]) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let marker = find_subslice(bytes, PAYLOAD_MARKER)
        .ok_or("not a LawSynth workspace archive (missing payload marker)")?;
    let header = std::str::from_utf8(&bytes[..marker])
        .map_err(|_| "workspace archive header is not valid UTF-8".to_owned())?;
    let mut lines = header.lines();
    if lines.next() != Some(MAGIC) {
        return Err("unsupported workspace archive format (expected LSWORKSPACE v1)".to_owned());
    }

    struct Manifest {
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
            return Err(format!("malformed workspace manifest line: {line}"));
        }
        let len = fields[1]
            .parse::<usize>()
            .map_err(|_| format!("invalid entry length: {}", fields[1]))?;
        manifest.push(Manifest { len, hash: fields[2].to_owned(), path: fields[3].to_owned() });
    }

    let mut cursor = marker + PAYLOAD_MARKER.len();
    let mut entries = BTreeMap::new();
    for entry in manifest {
        let end = cursor
            .checked_add(entry.len)
            .filter(|end| *end <= bytes.len())
            .ok_or_else(|| format!("workspace archive is truncated at {}", entry.path))?;
        let content = bytes[cursor..end].to_vec();
        let actual = sha256_hex(&content);
        if actual != entry.hash {
            return Err(format!(
                "integrity check failed for {}: expected {}, got {}",
                entry.path,
                short_hash(&entry.hash),
                short_hash(&actual)
            ));
        }
        entries.insert(entry.path, content);
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

fn workspace_dir(dir_override: Option<&str>) -> Result<PathBuf, String> {
    match dir_override {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => {
            let home = std::env::var("HOME").map_err(|_| {
                "HOME is not set; pass --dir to choose a workspace directory".to_owned()
            })?;
            Ok(PathBuf::from(home).join(".lawsynth"))
        }
    }
}

fn extract_flags(arguments: &[String]) -> Result<(Option<String>, bool, Vec<String>), String> {
    let mut dir = None;
    let mut force = false;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--dir" => {
                let value =
                    arguments.get(index + 1).ok_or_else(|| "missing value for --dir".to_owned())?;
                dir = Some(value.clone());
                index += 2;
            }
            "--force" => {
                force = true;
                index += 1;
            }
            _ => {
                rest.push(arguments[index].clone());
                index += 1;
            }
        }
    }
    Ok((dir, force, rest))
}

/// Maps a library name to a filesystem-safe bundle filename stem.
fn safe_filename(name: &str) -> String {
    let mapped: String = name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == '.' { ch } else { '_' }
        })
        .collect();
    if mapped.is_empty() { "world".to_owned() } else { mapped }
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn short_hash(hash: &str) -> String {
    if hash.len() > 12 { format!("{}…", &hash[..12]) } else { hash.to_owned() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn container_round_trips_with_integrity() {
        let mut entries = BTreeMap::new();
        entries.insert("library.tsv".to_owned(), b"name\tpath\nalpha\t/a".to_vec());
        entries.insert("worlds/0000.lsworld".to_owned(), vec![0, 1, 2, 3, 255]);
        let container = write_container(&entries);
        let decoded = read_container(&container).unwrap();
        assert_eq!(decoded, entries);
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let mut entries = BTreeMap::new();
        entries.insert("a.bin".to_owned(), vec![1, 2, 3]);
        let mut container = write_container(&entries);
        // Flip a payload byte (the last byte belongs to the payload region).
        let last = container.len() - 1;
        container[last] ^= 0xff;
        assert!(read_container(&container).is_err());
    }

    #[test]
    fn library_lines_skips_header_and_blanks() {
        let bytes = b"name\tpath\ttags\nalpha\t/a\tx,y\n\nbeta\t/b\t";
        let lines = library_lines(bytes);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].0, "alpha");
        assert_eq!(lines[0].1[1], "/a");
        assert_eq!(lines[1].0, "beta");
    }

    #[test]
    fn safe_filename_strips_unsafe_characters() {
        assert_eq!(safe_filename("predator/prey"), "predator_prey");
        assert_eq!(safe_filename("lotka-volterra.v2"), "lotka-volterra.v2");
    }

    #[test]
    fn find_subslice_locates_marker() {
        assert_eq!(find_subslice(b"abc--payload--\nxyz", PAYLOAD_MARKER), Some(3));
        assert_eq!(find_subslice(b"no marker here", PAYLOAD_MARKER), None);
    }
}
