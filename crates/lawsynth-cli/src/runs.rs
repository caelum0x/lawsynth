//! `lawsynth runs` — deterministic experiment / run tracking.
//!
//! A "run" is a single `discover` invocation captured as a content-addressed
//! record: the input data's SHA-256 hash + column set, the discovery
//! configuration (preset, degree, threshold, solver, derivative, feature and
//! pass toggles), and the result summary (mse, complexity, law count, Pareto
//! size, regime segments). Records live under a workspace directory (default
//! `~/.lawsynth/runs/`, override with `--dir`), one human-readable `<id>.run`
//! file per run.
//!
//! ## No wall clock
//!
//! The environment is clock-free, so a run's identity is *derived from its
//! content*, never from a timestamp: the id is the first 12 hex characters of
//! `sha256(label + data hash + canonical config)`. Re-running the same
//! configuration on the same data (with the same label) is idempotent — it
//! resolves to the same id and overwrites in place. Changing any tracked knob
//! (preset, threshold, degree, solver, a toggle, the data, or the label)
//! produces a different id. Listings are ordered lexicographically by id, so
//! output is stable across machines and runs.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::sha256_hex;

/// A recorded discovery run as an ordered list of dotted key/value fields.
///
/// Keys are namespaced: `label`, `data.*`, `config.*`, and `result.*`. Order is
/// preserved for stable, human-readable serialization; the `config.*`, `label`,
/// and `data.hash` fields feed the content-derived id.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunRecord {
    pub id: String,
    pub fields: Vec<(String, String)>,
}

const RECORD_HEADER: &str = "# lawsynth run record v1";
const ID_WIDTH: usize = 12;

impl RunRecord {
    /// Returns the value of a dotted field key, if present.
    fn get(&self, key: &str) -> Option<&str> {
        self.fields.iter().find(|(name, _)| name == key).map(|(_, value)| value.as_str())
    }

    /// Computes the content-derived id from label, data hash, and config fields.
    ///
    /// Deliberately excludes `result.*` and volatile `data.samples` so the id
    /// tracks the *experiment* (inputs + configuration), not its outcome.
    fn derive_id(&self) -> String {
        let mut canonical = String::new();
        for (key, value) in &self.fields {
            if key == "label" || key == "data.hash" || key.starts_with("config.") {
                let _ = writeln!(canonical, "{key}={value}");
            }
        }
        let digest = sha256_hex(canonical.as_bytes());
        digest[..ID_WIDTH].to_owned()
    }
}

/// Builder that assembles a [`RunRecord`] from a discovery invocation.
///
/// The CLI's `discover --track` path fills this in; keeping it here means the
/// on-disk field order and id derivation live in one place.
#[derive(Clone, Debug, Default)]
pub struct RunBuilder {
    fields: Vec<(String, String)>,
}

impl RunBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a field, sanitizing the value so it survives the line format.
    pub fn field(mut self, key: &str, value: impl Into<String>) -> Self {
        self.fields.push((key.to_owned(), sanitize(&value.into())));
        self
    }

    /// Appends a boolean toggle as `true`/`false`.
    pub fn toggle(self, key: &str, on: bool) -> Self {
        self.field(key, if on { "true" } else { "false" })
    }

    /// Finalizes the record, deriving and stamping its content id.
    pub fn build(self) -> RunRecord {
        let mut record = RunRecord { id: String::new(), fields: self.fields };
        record.id = record.derive_id();
        record
    }
}

/// Persists a run record and returns a short confirmation message.
pub fn record_run(dir_override: Option<&str>, record: &RunRecord) -> Result<String, String> {
    let directory = runs_dir(dir_override)?;
    fs::create_dir_all(&directory)
        .map_err(|error| format!("failed to create {}: {error}", directory.display()))?;
    let path = directory.join(format!("{}.run", record.id));
    fs::write(&path, serialize(record))
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    let label = record.get("label").unwrap_or("-");
    Ok(format!("tracked run {} (label: {})\n", record.id, label))
}

/// Runs the `runs` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    let Some(subcommand) = arguments.first().map(String::as_str) else {
        return Err(help());
    };
    if subcommand == "--help" || subcommand == "-h" {
        return Ok(help());
    }
    let (dir_override, rest) = extract_dir(&arguments[1..])?;
    match subcommand {
        "list" => list(dir_override.as_deref(), &rest),
        "show" => show(dir_override.as_deref(), &rest),
        "compare" => compare(dir_override.as_deref(), &rest),
        _ => Err(help()),
    }
}

/// Help text for `lawsynth runs`.
pub fn help() -> String {
    "lawsynth runs <list|show|compare> [--dir DIR] ...\n\n\
  runs list                         list tracked discovery runs (ordered by id)\n\
  runs show ID                      show one run's config + result in full\n\
  runs compare ID-A ID-B            diff two runs' config and result\n\n\
Runs are recorded by `lawsynth discover ... --track [--label TEXT]`. Each run's \
id is derived from its data hash + configuration (never a wall clock), so the \
same experiment resolves to the same id. Records default to ~/.lawsynth/runs/; \
override the directory with --dir."
        .to_owned()
}

fn list(dir_override: Option<&str>, arguments: &[String]) -> Result<String, String> {
    if !arguments.is_empty() {
        return Err(help());
    }
    let directory = runs_dir(dir_override)?;
    let records = load_all(&directory)?;
    if records.is_empty() {
        return Ok(format!("no tracked runs in {}\n", directory.display()));
    }
    let mut out = String::new();
    let _ = writeln!(out, "{} run(s) in {}", records.len(), directory.display());
    let _ = writeln!(
        out,
        "  {:<12}  {:<16}  {:<6}  {:<12}  {:<10}  mse",
        "id", "label", "degree", "thresh", "complexity"
    );
    for record in &records {
        let _ = writeln!(
            out,
            "  {:<12}  {:<16}  {:<6}  {:<12}  {:<10}  {}",
            record.id,
            truncate(record.get("label").unwrap_or("-"), 16),
            record.get("config.degree").unwrap_or("-"),
            record.get("config.threshold").unwrap_or("-"),
            record.get("result.complexity").unwrap_or("-"),
            record.get("result.mse").unwrap_or("-"),
        );
    }
    Ok(out)
}

fn show(dir_override: Option<&str>, arguments: &[String]) -> Result<String, String> {
    let Some(id) = arguments.first() else {
        return Err("usage: runs show ID".to_owned());
    };
    let directory = runs_dir(dir_override)?;
    let record = load_one(&directory, id)?;
    let mut out = String::new();
    let _ = writeln!(out, "run {}", record.id);
    let width = record.fields.iter().map(|(key, _)| key.len()).max().unwrap_or(4);
    for (key, value) in &record.fields {
        let _ = writeln!(out, "  {key:<width$}  {value}", width = width);
    }
    Ok(out)
}

fn compare(dir_override: Option<&str>, arguments: &[String]) -> Result<String, String> {
    let (Some(id_a), Some(id_b)) = (arguments.first(), arguments.get(1)) else {
        return Err("usage: runs compare ID-A ID-B".to_owned());
    };
    if arguments.len() > 2 {
        return Err("usage: runs compare ID-A ID-B".to_owned());
    }
    let directory = runs_dir(dir_override)?;
    let a = load_one(&directory, id_a)?;
    let b = load_one(&directory, id_b)?;

    let mut out = String::new();
    let _ = writeln!(out, "comparing {} (A) vs {} (B)", a.id, b.id);

    let keys = union_keys(&a, &b);
    let config_diffs: Vec<&String> = keys
        .iter()
        .filter(|key| {
            (key.starts_with("config.") || key.starts_with("data.")) && differs(&a, &b, key)
        })
        .collect();
    let _ = writeln!(out, "\nconfig deltas:");
    if config_diffs.is_empty() {
        let _ = writeln!(out, "  (identical configuration)");
    } else {
        for key in config_diffs {
            let _ = writeln!(
                out,
                "  {key:<20}  A={}  B={}",
                a.get(key).unwrap_or("-"),
                b.get(key).unwrap_or("-")
            );
        }
    }

    let _ = writeln!(out, "\nresult deltas:");
    for key in keys.iter().filter(|key| key.starts_with("result.")) {
        let va = a.get(key).unwrap_or("-");
        let vb = b.get(key).unwrap_or("-");
        let delta = numeric_delta(va, vb);
        let _ = writeln!(out, "  {key:<20}  A={va:<14}  B={vb:<14}{delta}");
    }
    Ok(out)
}

/// Renders the signed A→B delta for numeric result fields; empty otherwise.
fn numeric_delta(a: &str, b: &str) -> String {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(va), Ok(vb)) => {
            let delta = vb - va;
            let sign = if delta > 0.0 { "+" } else { "" };
            format!("  Δ={sign}{delta:.6e}")
        }
        _ => String::new(),
    }
}

fn differs(a: &RunRecord, b: &RunRecord, key: &str) -> bool {
    a.get(key) != b.get(key)
}

fn union_keys(a: &RunRecord, b: &RunRecord) -> Vec<String> {
    let mut keys: Vec<String> = a.fields.iter().map(|(key, _)| key.clone()).collect();
    for (key, _) in &b.fields {
        if !keys.contains(key) {
            keys.push(key.clone());
        }
    }
    keys
}

fn serialize(record: &RunRecord) -> String {
    let mut out = String::from(RECORD_HEADER);
    out.push('\n');
    let _ = writeln!(out, "id\t{}", record.id);
    for (key, value) in &record.fields {
        let _ = writeln!(out, "{key}\t{value}");
    }
    out
}

fn parse(contents: &str, fallback_id: &str) -> RunRecord {
    let mut id = fallback_id.to_owned();
    let mut fields = Vec::new();
    for line in contents.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('\t') else {
            continue;
        };
        if key == "id" {
            id = value.to_owned();
        } else {
            fields.push((key.to_owned(), value.to_owned()));
        }
    }
    RunRecord { id, fields }
}

fn load_one(directory: &Path, id: &str) -> Result<RunRecord, String> {
    let path = directory.join(format!("{id}.run"));
    let contents = fs::read_to_string(&path)
        .map_err(|_| format!("no tracked run '{id}' in {}", directory.display()))?;
    Ok(parse(&contents, id))
}

fn load_all(directory: &Path) -> Result<Vec<RunRecord>, String> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("failed to read {}: {error}", directory.display())),
    };
    let mut records = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("run") {
            continue;
        }
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_owned();
        let contents = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        records.push(parse(&contents, &stem));
    }
    // Deterministic ordering by content-derived id — never by file mtime.
    records.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(records)
}

fn runs_dir(dir_override: Option<&str>) -> Result<PathBuf, String> {
    match dir_override {
        Some(dir) => Ok(PathBuf::from(dir)),
        None => {
            let home = std::env::var("HOME")
                .map_err(|_| "HOME is not set; pass --dir to choose a runs directory".to_owned())?;
            Ok(PathBuf::from(home).join(".lawsynth").join("runs"))
        }
    }
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

fn sanitize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

fn truncate(value: &str, width: usize) -> String {
    if value.chars().count() > width {
        let mut truncated: String = value.chars().take(width.saturating_sub(1)).collect();
        truncated.push('…');
        truncated
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_builder(label: &str, threshold: &str) -> RunBuilder {
        RunBuilder::new()
            .field("label", label)
            .field("data.hash", "cafebabe")
            .field("data.columns", "time,x,y")
            .field("data.samples", "64")
            .field("config.preset", "default")
            .field("config.degree", "2")
            .field("config.threshold", threshold)
            .field("config.solver", "stlsq")
            .toggle("config.trigonometric", false)
            .field("result.mse", "1.234560e-6")
            .field("result.complexity", "8")
    }

    #[test]
    fn id_is_stable_for_identical_config_and_ignores_results() {
        let a = sample_builder("run-a", "5.000000e-2").build();
        // Same config + label but a different result must yield the same id.
        let b = sample_builder("run-a", "5.000000e-2").field("result.mse", "9.999999e-1").build();
        assert_eq!(a.id, b.id);
        assert_eq!(a.id.len(), ID_WIDTH);
    }

    #[test]
    fn id_changes_when_config_changes() {
        let a = sample_builder("run-a", "5.000000e-2").build();
        let b = sample_builder("run-a", "1.000000e-1").build();
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn id_changes_when_label_changes() {
        let a = sample_builder("run-a", "5.000000e-2").build();
        let b = sample_builder("run-b", "5.000000e-2").build();
        assert_ne!(a.id, b.id);
    }

    #[test]
    fn round_trips_through_the_on_disk_format() {
        let record = sample_builder("run-a", "5.000000e-2").build();
        let parsed = parse(&serialize(&record), "wrong-fallback");
        assert_eq!(parsed.id, record.id);
        assert_eq!(parsed.fields, record.fields);
        assert_eq!(parsed.get("config.degree"), Some("2"));
    }

    #[test]
    fn records_and_lists_deterministically() {
        let dir = std::env::temp_dir().join(format!("lawsynth-runs-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let directory = dir.to_string_lossy().into_owned();
        let a = sample_builder("alpha", "5.000000e-2").build();
        let b = sample_builder("beta", "1.000000e-1").build();
        record_run(Some(&directory), &a).unwrap();
        record_run(Some(&directory), &b).unwrap();
        // Idempotent: recording the same run again overwrites, no duplicate.
        record_run(Some(&directory), &a).unwrap();
        let listed = load_all(&dir).unwrap();
        assert_eq!(listed.len(), 2);
        assert!(listed[0].id <= listed[1].id);
        let _ = fs::remove_dir_all(&dir);
    }
}
