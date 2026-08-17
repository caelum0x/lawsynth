//! `lawsynth library` — a local, human-readable world library index.
//!
//! The index is a tab-separated file (default `~/.lawsynth/library.tsv`) mapping
//! a unique name to a bundle path, tags, and a description. All writes are
//! deterministic (entries sorted by name) and non-clobbering.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::read_world;

const HEADER: &str = "name\tpath\ttags\tdescription";

/// Help text for `lawsynth library`.
pub fn help() -> String {
    "lawsynth library <add|list|show|remove> [--dir DIR] ...\n\n\
  library add NAME PATH.lsworld [--tags a,b,c] [--description TEXT]\n\
  library list\n\
  library show NAME\n\
  library remove NAME\n\n\
The index defaults to ~/.lawsynth/library.tsv; override the directory with --dir."
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
        "remove" => remove(&index_path, &rest),
        _ => Err(help()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    name: String,
    path: String,
    tags: Vec<String>,
    description: String,
}

fn add(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    let (Some(name), Some(path)) = (arguments.first(), arguments.get(1)) else {
        return Err(
            "usage: library add NAME PATH.lsworld [--tags a,b] [--description TEXT]".to_owned()
        );
    };
    validate_name(name)?;
    if !Path::new(path).exists() {
        return Err(format!("bundle path does not exist: {path}"));
    }
    let mut tags = Vec::new();
    let mut description = String::new();
    let mut index = 2;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--tags" => {
                tags = value
                    .split(',')
                    .map(|tag| tag.trim().to_owned())
                    .filter(|tag| !tag.is_empty())
                    .collect()
            }
            "--description" => description = sanitize(value),
            _ => return Err(help()),
        }
        index += 2;
    }

    let mut entries = load(index_path)?;
    if entries.iter().any(|entry| entry.name == *name) {
        return Err(format!("library already has an entry named '{name}' (remove it first)"));
    }
    entries.push(Entry { name: name.clone(), path: path.clone(), tags, description });
    save(index_path, &entries)?;
    Ok(format!("added '{name}' -> {path}\n"))
}

fn list(index_path: &Path, arguments: &[String]) -> Result<String, String> {
    if !arguments.is_empty() {
        return Err(help());
    }
    let entries = load(index_path)?;
    if entries.is_empty() {
        return Ok(format!("library is empty ({})\n", index_path.display()));
    }
    let width = entries.iter().map(|entry| entry.name.len()).max().unwrap_or(4).max(4);
    let mut out = String::new();
    let _ = writeln!(out, "{} entr(y/ies) in {}", entries.len(), index_path.display());
    for entry in &entries {
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
    let _ = writeln!(
        out,
        "description: {}",
        if entry.description.is_empty() { "-".to_owned() } else { entry.description.clone() }
    );
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
        if line.is_empty() || line == HEADER {
            continue;
        }
        let fields: Vec<&str> = line.splitn(4, '\t').collect();
        if fields.len() < 2 {
            continue;
        }
        entries.push(Entry {
            name: fields[0].to_owned(),
            path: fields[1].to_owned(),
            tags: fields
                .get(2)
                .map(|tags| {
                    tags.split(',')
                        .map(|tag| tag.to_owned())
                        .filter(|tag| !tag.is_empty())
                        .collect()
                })
                .unwrap_or_default(),
            description: fields.get(3).map(|value| (*value).to_owned()).unwrap_or_default(),
        });
    }
    Ok(entries)
}

fn save(index_path: &Path, entries: &[Entry]) -> Result<(), String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|left, right| left.name.cmp(&right.name));
    let mut contents = String::from(HEADER);
    contents.push('\n');
    for entry in &sorted {
        let _ = writeln!(
            contents,
            "{}\t{}\t{}\t{}",
            entry.name,
            entry.path,
            entry.tags.join(","),
            entry.description
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
            },
            Entry {
                name: "alpha".to_owned(),
                path: "/tmp/a.lsworld".to_owned(),
                tags: vec![],
                description: "first".to_owned(),
            },
        ];
        save(&index, &entries).unwrap();
        let loaded = load(&index).unwrap();
        // Sorted deterministically by name on save.
        assert_eq!(loaded[0].name, "alpha");
        assert_eq!(loaded[1].tags, vec!["x", "y"]);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_index_is_empty() {
        let index = std::env::temp_dir().join("lawsynth-nonexistent-xyz/library.tsv");
        assert!(load(&index).unwrap().is_empty());
    }
}
