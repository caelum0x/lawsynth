//! `lawsynth library` — a local, provenance-aware world registry.
//!
//! The index is a tab-separated file (default `~/.lawsynth/library.tsv`) mapping
//! a unique name to a bundle path, tags, a description, and provenance: a content
//! hash of the `.lsworld`, the optional source data's hash + column set, and an
//! optional config summary. All writes are deterministic (entries sorted by name)
//! and non-clobbering. The format is backward-compatible: entries written by the
//! earlier four-column layout still load, with empty provenance.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::{read_world, sha256_hex};

/// TSV header for the library index. Shared with `workspace` so exported and
/// imported indexes stay byte-compatible with `library`'s own writer.
pub(crate) const HEADER: &str =
    "name\tpath\ttags\tdescription\tworld_hash\tdata_hash\tdata_columns\tconfig";

/// Help text for `lawsynth library`.
pub fn help() -> String {
    "lawsynth library <add|list|show|search|compare|remove> [--dir DIR] ...\n\n\
  library add WORLD.lsworld --name N [--tags a,b,c] [--from-data OBS.csv] [--config TEXT] [--note TEXT]\n\
  library add NAME WORLD.lsworld [--tags a,b,c] [--description TEXT]   (legacy positional form)\n\
  library list\n\
  library show NAME\n\
  library search QUERY                 (matches name, tags, and description)\n\
  library compare NAME-A NAME-B [--json] [--html FILE]\n\
  library remove NAME\n\n\
`add` captures provenance: a SHA-256 content hash of the bundle, and — with \
--from-data — the source data's SHA-256 hash and column set. The index defaults \
to ~/.lawsynth/library.tsv; override the directory with --dir."
        .to_owned()
}

/// Runs the `library` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(subcommand) = arguments.first().map(String::as_str) else {
        return Err(help());
    };
    if subcommand == "--help" || subcommand == "-h" {
        return Ok(help());
    }
    // Extract the shared --dir option, keeping the remaining positional/flag args.
    let (dir_override, rest) = extract_dir(&arguments[1..])?;
    let index_path = index_path(dir_override.as_deref())?;

    match subcommand {
        "add" => add(&index_path, &rest),
        "list" => list(&index_path, &rest),
        "show" => show(&index_path, &rest),
        "search" => search(&index_path, &rest),
        "compare" => compare(&index_path, &rest),
        "remove" => remove(&index_path, &rest),
        _ => Err(help()),
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Entry {
    name: String,
    path: String,
    tags: Vec<String>,
    description: String,
    /// SHA-256 of the `.lsworld` bundle bytes at registration time.
    world_hash: String,
    /// SHA-256 of the source observation file, when registered with --from-data.
    data_hash: String,
    /// Comma-joined column identifiers of the source observation file.
    data_columns: String,
    /// Free-text configuration summary captured with --config.
    config_summary: String,
}

/// Parsed `add` arguments, resolved from either the new or legacy form.
struct AddArgs {
    name: String,
    path: String,
    tags: Vec<String>,
    description: String,
    from_data: Option<String>,
    config: Option<String>,
}

fn parse_add(arguments: &[String]) -> Result<AddArgs, String> {
    let mut positionals = Vec::new();
    let mut name_flag = None;
    let mut tags = Vec::new();
    let mut description = String::new();
    let mut from_data = None;
    let mut config = None;
    let mut index = 0;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        if option.starts_with("--") {
            let value =
                arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
            match option {
                "--name" => name_flag = Some(value.clone()),
                "--tags" => tags = parse_tags(value),
                "--note" | "--description" => description = sanitize(value),
                "--from-data" => from_data = Some(value.clone()),
                "--config" => config = Some(sanitize(value)),
                _ => return Err(help()),
            }
            index += 2;
        } else {
            positionals.push(arguments[index].clone());
            index += 1;
        }
    }

    // New form: `add WORLD --name N`. Legacy form: `add NAME WORLD`.
    let (name, path) = match name_flag {
        Some(name) => {
            let path = positionals.first().ok_or_else(|| {
                "usage: library add WORLD.lsworld --name N [--tags a,b] [--from-data OBS.csv] [--config TEXT] [--note TEXT]".to_owned()
            })?;
            (name, path.clone())
        }
        None => {
            let (Some(name), Some(path)) = (positionals.first(), positionals.get(1)) else {
                return Err(
                    "usage: library add WORLD.lsworld --name N [--tags a,b] [--from-data OBS.csv] [--config TEXT] [--note TEXT]".to_owned(),
                );
            };
            (name.clone(), path.clone())
        }
    };
    Ok(AddArgs { name, path, tags, description, from_data, config })
}

fn add(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let args = parse_add(arguments)?;
    validate_name(&args.name)?;
    if !Path::new(&args.path).exists() {
        return Err(format!("bundle path does not exist: {}", args.path));
    }
    // Content hash of the bundle: the provenance anchor.
    let world_bytes =
        fs::read(&args.path).map_err(|error| format!("failed to read {}: {error}", args.path))?;
    let world_hash = sha256_hex(&world_bytes);

    // Optional source-data provenance: hash + column set.
    let (data_hash, data_columns) = match &args.from_data {
        Some(data_path) => {
            let bytes = fs::read(data_path)
                .map_err(|error| format!("failed to read {data_path}: {error}"))?;
            (sha256_hex(&bytes), header_columns(data_path, &bytes))
        }
        None => (String::new(), String::new()),
    };

    let mut entries = load(index_path)?;
    if entries.iter().any(|entry| entry.name == args.name) {
        return Err(format!(
            "library already has an entry named '{}' (remove it first)",
            args.name
        ));
    }
    entries.push(Entry {
        name: args.name.clone(),
        path: args.path.clone(),
        tags: args.tags,
        description: args.description,
        world_hash: world_hash.clone(),
        data_hash,
        data_columns,
        config_summary: args.config.unwrap_or_default(),
    });
    save(index_path, &entries)?;
    let mut out = format!("added '{}' -> {}\n", args.name, args.path);
    let _ = writeln!(out, "  world hash: {}", short_hash(&world_hash));
    if let Some(data_path) = &args.from_data {
        let _ = writeln!(out, "  from data:  {data_path}");
    }
    Ok(out)
}

fn list(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    if !arguments.is_empty() {
        return Err(help());
    }
    let entries = load(index_path)?;
    render_list(&entries, index_path, "entr(y/ies)")
}

fn search(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let Some(query) = arguments.first() else {
        return Err("usage: library search QUERY".to_owned());
    };
    if arguments.len() > 1 {
        return Err("usage: library search QUERY".to_owned());
    }
    let needle = query.to_lowercase();
    let entries = load(index_path)?;
    let matches: Vec<Entry> =
        entries.into_iter().filter(|entry| entry_matches(entry, &needle)).collect();
    if matches.is_empty() {
        return Ok(format!("no matches for '{query}' in {}\n", index_path.display()));
    }
    render_list(&matches, index_path, &format!("match(es) for '{query}'"))
}

/// Case-insensitive substring match across name, tags, and description.
fn entry_matches(entry: &Entry, needle: &str) -> bool {
    entry.name.to_lowercase().contains(needle)
        || entry.description.to_lowercase().contains(needle)
        || entry.tags.iter().any(|tag| tag.to_lowercase().contains(needle))
}

fn render_list(entries: &[Entry], index_path: &Path, noun: &str) -> Result<String, String> {
    if entries.is_empty() {
        return Ok(format!("library is empty ({})\n", index_path.display()));
    }
    let width = entries.iter().map(|entry| entry.name.len()).max().unwrap_or(4).max(4);
    let mut out = String::new();
    let _ = writeln!(out, "{} {noun} in {}", entries.len(), index_path.display());
    for entry in entries {
        let tags = if entry.tags.is_empty() {
            String::new()
        } else {
            format!("  [{}]", entry.tags.join(", "))
        };
        let _ = writeln!(out, "  {:<width$}  {}{}", entry.name, entry.path, tags, width = width);
    }
    Ok(out)
}

fn show(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let Some(name) = arguments.first() else {
        return Err("usage: library show NAME".to_owned());
    };
    let entries = load(index_path)?;
    let entry = entries
        .iter()
        .find(|entry| entry.name == *name)
        .ok_or_else(|| format!("no library entry named '{name}'"))?;
    let mut out = String::new();
    let _ = writeln!(out, "name:        {}", entry.name);
    let _ = writeln!(out, "path:        {}", entry.path);
    let _ = writeln!(
        out,
        "tags:        {}",
        if entry.tags.is_empty() { "-".to_owned() } else { entry.tags.join(", ") }
    );
    let _ = writeln!(out, "description: {}", dash_if_empty(&entry.description));
    let _ = writeln!(out, "world hash:  {}", dash_if_empty(&entry.world_hash));
    let _ = writeln!(out, "data hash:   {}", dash_if_empty(&entry.data_hash));
    let _ = writeln!(out, "data cols:   {}", dash_if_empty(&entry.data_columns));
    let _ = writeln!(out, "config:      {}", dash_if_empty(&entry.config_summary));
    match read_world(&entry.path) {
        Ok(world) => {
            let _ = writeln!(
                out,
                "world:       {} state(s), {} variable(s), {} parameter(s)",
                world.state_ids().count(),
                world.variables().len(),
                world.parameters().len()
            );
        }
        Err(error) => {
            let _ = writeln!(out, "world:       <unreadable: {error}>");
        }
    }
    Ok(out)
}

/// `library compare A B [flags]` — resolves two registered names to their bundle
/// paths and delegates to the `compare` command, forwarding any extra flags.
fn compare(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let (Some(name_a), Some(name_b)) = (arguments.first(), arguments.get(1)) else {
        return Err("usage: library compare NAME-A NAME-B [--json] [--html FILE]".to_owned());
    };
    let entries = load(index_path)?;
    let path_a = resolve_path(&entries, name_a)?;
    let path_b = resolve_path(&entries, name_b)?;
    let mut forwarded = vec![path_a, path_b];
    forwarded.extend_from_slice(&arguments[2..]);
    crate::compare::run(&forwarded)
}

fn resolve_path(entries: &[Entry], name: &str) -> Result<String, String> {
    entries
        .iter()
        .find(|entry| entry.name == name)
        .map(|entry| entry.path.clone())
        .ok_or_else(|| format!("no library entry named '{name}'"))
}

fn remove(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let Some(name) = arguments.first() else {
        return Err("usage: library remove NAME".to_owned());
    };
    let mut entries = load(index_path)?;
    let before = entries.len();
    entries.retain(|entry| entry.name != *name);
    if entries.len() == before {
        return Err(format!("no library entry named '{name}'"));
    }
    save(index_path, &entries)?;
    Ok(format!("removed '{name}'\n"))
}

fn extract_dir(arguments: &[String]) -> Result<(Option<String>, Vec<String>), String> {
    let mut dir = None;
    let mut rest = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        if arguments[index] == "--dir" {
            let value =
                arguments.get(index + 1).ok_or_else(|| "missing value for --dir".to_owned())?;
            dir = Some(value.clone());
            index += 2;
        } else {
            rest.push(arguments[index].clone());
            index += 1;
        }
    }
    Ok((dir, rest))
}

fn index_path(dir_override: Option<&str>) -> Result<PathBuf, String> {
    let directory = match dir_override {
        Some(dir) => PathBuf::from(dir),
        None => {
            let home = std::env::var("HOME").map_err(|_| {
                "HOME is not set; pass --dir to choose a library directory".to_owned()
            })?;
            PathBuf::from(home).join(".lawsynth")
        }
    };
    Ok(directory.join("library.tsv"))
}

fn load(index_path: &Path) -> Result<Vec<Entry>, String> {
    let contents = match fs::read_to_string(index_path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", index_path.display())),
    };
    let mut entries = Vec::new();
    for line in contents.lines() {
        // Skip blanks and any header line (old four-column or new eight-column).
        if line.is_empty() || line.starts_with("name\tpath\t") {
            continue;
        }
        let fields: Vec<&str> = line.splitn(8, '\t').collect();
        if fields.len() < 2 {
            continue;
        }
        entries.push(Entry {
            name: fields[0].to_owned(),
            path: fields[1].to_owned(),
            tags: fields.get(2).map(|tags| parse_tags(tags)).unwrap_or_default(),
            description: field(&fields, 3),
            world_hash: field(&fields, 4),
            data_hash: field(&fields, 5),
            data_columns: field(&fields, 6),
            config_summary: field(&fields, 7),
        });
    }
    Ok(entries)
}

fn field(fields: &[&str], index: usize) -> String {
    fields.get(index).map(|value| (*value).to_owned()).unwrap_or_default()
}

fn save(index_path: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));
    let mut contents = String::from(HEADER);
    contents.push('\n');
    for entry in &sorted {
        let _ = writeln!(
            contents,
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            entry.name,
            entry.path,
            entry.tags.join(","),
            entry.description,
            entry.world_hash,
            entry.data_hash,
            entry.data_columns,
            entry.config_summary,
        );
    }
    if let Some(parent) = index_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }
    fs::write(index_path, contents)
        .map_err(|error| format!("failed to write {}: {error}", index_path.display()))
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("name must not be empty".to_owned());
    }
    if name.contains('\t') || name.contains('\n') || name.starts_with("--") {
        return Err(format!("invalid library name '{name}'"));
    }
    Ok(())
}

fn parse_tags(value: &str) -> Vec<String> {
    value.split(',').map(|tag| tag.trim().to_owned()).filter(|tag| !tag.is_empty()).collect()
}

/// Reads the header row of a delimited observation file and returns its column
/// identifiers, comma-joined. TSV files split on tabs, everything else on commas.
fn header_columns(path: &str, bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    let Some(header) = text.lines().next() else {
        return String::new();
    };
    let delimiter = if path.to_ascii_lowercase().ends_with(".tsv") { '\t' } else { ',' };
    let columns: Vec<String> = header
        .split(delimiter)
        .map(|column| column.trim().to_owned())
        .filter(|column| !column.is_empty())
        .collect();
    sanitize(&columns.join(","))
}

fn short_hash(hash: &str) -> String {
    if hash.len() > 12 { format!("{}…", &hash[..12]) } else { hash.to_owned() }
}

fn dash_if_empty(value: &str) -> String {
    if value.is_empty() { "-".to_owned() } else { value.to_owned() }
}

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_entries_through_tsv() {
        let dir = std::env::temp_dir().join(format!("lawsynth-lib-test-{}", std::process::id()));
        let index = dir.join("library.tsv");
        let _ = fs::remove_dir_all(&dir);
        let entries = vec![
            Entry {
                name: "beta".to_owned(),
                path: "/tmp/b.lsworld".to_owned(),
                tags: vec!["x".to_owned(), "y".to_owned()],
                description: "second".to_owned(),
                world_hash: "deadbeef".to_owned(),
                data_hash: "cafe".to_owned(),
                data_columns: "time,x,y".to_owned(),
                config_summary: "degree 2".to_owned(),
            },
            Entry {
                name: "alpha".to_owned(),
                path: "/tmp/a.lsworld".to_owned(),
                ..Entry::default()
            },
        ];
        save(&index, &entries).unwrap();
        let loaded = load(&index).unwrap();
        // Sorted deterministically by name on save.
        assert_eq!(loaded[0].name, "alpha");
        assert_eq!(loaded[1].tags, vec!["x", "y"]);
        assert_eq!(loaded[1].world_hash, "deadbeef");
        assert_eq!(loaded[1].data_columns, "time,x,y");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn loads_legacy_four_column_entries() {
        let dir = std::env::temp_dir().join(format!("lawsynth-lib-legacy-{}", std::process::id()));
        let index = dir.join("library.tsv");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            &index,
            "name\tpath\ttags\tdescription\nold\t/tmp/old.lsworld\ta,b\tan old entry\n",
        )
        .unwrap();
        let loaded = load(&index).unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "old");
        assert_eq!(loaded[0].tags, vec!["a", "b"]);
        assert!(loaded[0].world_hash.is_empty());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn search_matches_name_tags_and_description() {
        let entry = Entry {
            name: "predator-prey".to_owned(),
            tags: vec!["ecology".to_owned()],
            description: "Lotka-Volterra".to_owned(),
            ..Entry::default()
        };
        assert!(entry_matches(&entry, "prey"));
        assert!(entry_matches(&entry, "ecology"));
        assert!(entry_matches(&entry, "volterra"));
        assert!(!entry_matches(&entry, "finance"));
    }

    #[test]
    fn header_columns_parses_csv_and_tsv() {
        assert_eq!(header_columns("obs.csv", b"time,x,y\n0,1,2\n"), "time,x,y");
        assert_eq!(header_columns("obs.tsv", b"time\tx\ty\n0\t1\t2\n"), "time,x,y");
    }

    #[test]
    fn missing_index_is_empty() {
        let index = std::env::temp_dir().join("lawsynth-nonexistent-xyz/library.tsv");
        assert!(load(&index).unwrap().is_empty());
    }
}
