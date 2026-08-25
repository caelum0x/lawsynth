use std::collections::BTreeMap;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use lawsynth_information_diffusion::{Activation, Cascade, Config, Edge, Input, Limits, analyze};

const MAX_INPUT_BYTES: u64 = 64 * 1024 * 1024;

#[derive(Debug)]
struct Options {
    nodes: PathBuf,
    edges: PathBuf,
    cascades: PathBuf,
    activations: PathBuf,
    seeds: PathBuf,
    blocked_nodes: Option<PathBuf>,
    output: PathBuf,
    horizon: usize,
    simulations: usize,
    random_seed: u64,
    transmission_multiplier: f64,
    max_runtime_ms: u64,
    overwrite: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = parse_args()?;
    ensure_output_is_distinct(&options)?;
    let input = load_input(&options)?;
    let config = Config {
        horizon: options.horizon,
        simulations: options.simulations,
        random_seed: options.random_seed,
        blocked_nodes: load_single_column_optional(options.blocked_nodes.as_deref())?,
        transmission_multiplier: options.transmission_multiplier,
        max_runtime: Duration::from_millis(options.max_runtime_ms),
    };
    let analysis = analyze(input, config, &Limits::default()).map_err(|error| error.to_string())?;
    if !analysis.verify_receipt() {
        return Err("internal receipt verification failed".into());
    }
    let mut report = analysis.to_json();
    report.push('\n');
    write_atomic(&options.output, report.as_bytes(), options.overwrite)?;
    println!("{}", options.output.display());
    Ok(())
}

fn ensure_output_is_distinct(options: &Options) -> Result<(), String> {
    let Ok(output) = fs::canonicalize(&options.output) else {
        return Ok(());
    };
    let mut inputs = vec![
        &options.nodes,
        &options.edges,
        &options.cascades,
        &options.activations,
        &options.seeds,
    ];
    if let Some(blocked) = &options.blocked_nodes {
        inputs.push(blocked);
    }
    for input in inputs {
        if fs::canonicalize(input).map_err(|error| format!("{}: {error}", input.display()))?
            == output
        {
            return Err(format!(
                "{}: output must not replace an input file",
                options.output.display()
            ));
        }
    }
    Ok(())
}

fn parse_args() -> Result<Options, String> {
    let mut values = BTreeMap::<String, String>::new();
    let mut overwrite = false;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "--version" => {
                println!("lawsynth-information-diffusion {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--overwrite" => {
                if overwrite {
                    return Err("--overwrite was supplied more than once".into());
                }
                overwrite = true;
            }
            flag if flag.starts_with("--") => {
                if !matches!(
                    flag,
                    "--nodes"
                        | "--edges"
                        | "--cascades"
                        | "--activations"
                        | "--seeds"
                        | "--blocked-nodes"
                        | "--output"
                        | "--horizon"
                        | "--simulations"
                        | "--seed"
                        | "--transmission-multiplier"
                        | "--max-runtime-ms"
                ) {
                    return Err(format!("unknown option: {flag}"));
                }
                let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
                if value.starts_with("--") {
                    return Err(format!("missing value for {flag}"));
                }
                if values.insert(flag.to_owned(), value).is_some() {
                    return Err(format!("{flag} was supplied more than once"));
                }
            }
            _ => return Err(format!("unexpected positional argument: {argument}")),
        }
    }

    let required_path = |name: &str| {
        values
            .get(name)
            .map(|value| PathBuf::from(value.as_str()))
            .ok_or_else(|| format!("missing required option {name}"))
    };
    Ok(Options {
        nodes: required_path("--nodes")?,
        edges: required_path("--edges")?,
        cascades: required_path("--cascades")?,
        activations: required_path("--activations")?,
        seeds: required_path("--seeds")?,
        blocked_nodes: values.get("--blocked-nodes").map(|value| PathBuf::from(value.as_str())),
        output: required_path("--output")?,
        horizon: parse_value(&values, "--horizon", 30usize)?,
        simulations: parse_value(&values, "--simulations", 1_000usize)?,
        random_seed: parse_value(&values, "--seed", 42u64)?,
        transmission_multiplier: parse_value(&values, "--transmission-multiplier", 0.75f64)?,
        max_runtime_ms: parse_value(&values, "--max-runtime-ms", 30_000u64)?,
        overwrite,
    })
}

fn parse_value<T>(values: &BTreeMap<String, String>, name: &str, default: T) -> Result<T, String>
where
    T: std::str::FromStr,
{
    values.get(name).map_or(Ok(default), |value| {
        value.parse::<T>().map_err(|_| format!("invalid value for {name}: {value}"))
    })
}

fn load_input(options: &Options) -> Result<Input, String> {
    let nodes = load_single_column(&options.nodes)?;
    let seeds = load_single_column(&options.seeds)?;
    let edges = read_tsv(&options.edges, &["source", "target"])?
        .into_iter()
        .map(|row| Edge { source: row[0].clone(), target: row[1].clone() })
        .collect();

    let mut cascades = Vec::new();
    let mut cascade_indexes = BTreeMap::new();
    for row in read_tsv(&options.cascades, &["cascade_id", "started_at", "observation_end_step"])? {
        let observation_end_step = row[2].parse::<usize>().map_err(|_| {
            format!(
                "{}: invalid observation_end_step for cascade {}",
                options.cascades.display(),
                row[0]
            )
        })?;
        let index = cascades.len();
        if cascade_indexes.insert(row[0].clone(), index).is_some() {
            return Err(format!("{}: duplicate cascade_id {}", options.cascades.display(), row[0]));
        }
        cascades.push(Cascade {
            cascade_id: row[0].clone(),
            started_at: row[1].clone(),
            observation_end_step,
            activations: Vec::new(),
        });
    }

    for row in read_tsv(&options.activations, &["cascade_id", "node_id", "step"])? {
        let step = row[2].parse::<usize>().map_err(|_| {
            format!(
                "{}: invalid activation step for cascade {}",
                options.activations.display(),
                row[0]
            )
        })?;
        let index = cascade_indexes.get(&row[0]).ok_or_else(|| {
            format!(
                "{}: activation references unknown cascade {}",
                options.activations.display(),
                row[0]
            )
        })?;
        cascades[*index].activations.push(Activation { node: row[1].clone(), step });
    }
    Ok(Input { nodes, edges, cascades, seeds })
}

fn load_single_column(path: &Path) -> Result<Vec<String>, String> {
    Ok(read_tsv(path, &["node_id"])?.into_iter().map(|row| row[0].clone()).collect())
}

fn load_single_column_optional(path: Option<&Path>) -> Result<Vec<String>, String> {
    path.map_or_else(|| Ok(Vec::new()), load_single_column)
}

fn read_tsv(path: &Path, expected_header: &[&str]) -> Result<Vec<Vec<String>>, String> {
    let contents = read_bounded(path)?;
    let mut lines = contents.lines();
    let header = lines.next().ok_or_else(|| format!("{}: empty TSV file", path.display()))?;
    let actual_header = header.strip_suffix('\r').unwrap_or(header).split('\t').collect::<Vec<_>>();
    if actual_header != expected_header {
        return Err(format!(
            "{}: expected TSV header `{}`",
            path.display(),
            expected_header.join("\\t")
        ));
    }
    let mut rows = Vec::new();
    for (offset, line) in lines.enumerate() {
        let line_number = offset + 2;
        let line = line.strip_suffix('\r').unwrap_or(line);
        if line.is_empty() {
            return Err(format!("{}:{line_number}: blank rows are not allowed", path.display()));
        }
        let fields = line.split('\t').map(str::trim).map(str::to_owned).collect::<Vec<_>>();
        if fields.len() != expected_header.len() {
            return Err(format!(
                "{}:{line_number}: expected {} columns, found {}",
                path.display(),
                expected_header.len(),
                fields.len()
            ));
        }
        if fields.iter().any(|field| field.is_empty()) {
            return Err(format!("{}:{line_number}: empty fields are not allowed", path.display()));
        }
        rows.push(fields);
    }
    if rows.is_empty() {
        return Err(format!("{}: at least one data row is required", path.display()));
    }
    Ok(rows)
}

fn read_bounded(path: &Path) -> Result<String, String> {
    let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let size = file.metadata().map_err(|error| format!("{}: {error}", path.display()))?.len();
    if size > MAX_INPUT_BYTES {
        return Err(format!("{}: input exceeds the 64 MiB file limit", path.display()));
    }
    let mut bytes = Vec::with_capacity(size as usize);
    file.take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    if bytes.len() as u64 > MAX_INPUT_BYTES {
        return Err(format!("{}: input exceeds the 64 MiB file limit", path.display()));
    }
    String::from_utf8(bytes).map_err(|_| format!("{}: input must be UTF-8", path.display()))
}

fn write_atomic(path: &Path, contents: &[u8], overwrite: bool) -> Result<(), String> {
    let parent = path
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    if !parent.is_dir() {
        return Err(format!("{}: output directory does not exist", parent.display()));
    }
    let file_name =
        path.file_name().ok_or_else(|| "output must name a file".to_owned())?.to_string_lossy();
    let mut temporary = None;
    for attempt in 0..100u32 {
        let candidate = parent.join(format!(".{file_name}.{}.{}.tmp", std::process::id(), attempt));
        let mut output_options = OpenOptions::new();
        output_options.write(true).create_new(true);
        #[cfg(unix)]
        output_options.mode(0o600);
        match output_options.open(&candidate) {
            Ok(file) => {
                temporary = Some((candidate, file));
                break;
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(format!("{}: cannot create temporary output: {error}", path.display()));
            }
        }
    }
    let (temporary_path, mut file) = temporary
        .ok_or_else(|| format!("{}: could not reserve a temporary output", path.display()))?;
    let result = (|| {
        file.write_all(contents).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        if overwrite {
            fs::rename(&temporary_path, path).map_err(|error| error.to_string())?;
        } else {
            fs::hard_link(&temporary_path, path).map_err(|error| {
                if error.kind() == std::io::ErrorKind::AlreadyExists {
                    "output already exists; pass --overwrite to replace it".to_owned()
                } else {
                    error.to_string()
                }
            })?;
            fs::remove_file(&temporary_path).map_err(|error| error.to_string())?;
        }
        File::open(parent)
            .and_then(|directory| directory.sync_all())
            .map_err(|error| error.to_string())?;
        Ok::<(), String>(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary_path);
    }
    result.map_err(|error| format!("{}: atomic write failed: {error}", path.display()))
}

fn print_help() {
    println!(
        "lawsynth-information-diffusion\n\nCalibrate and forecast observed information cascades.\n\nRequired:\n  --nodes PATH          TSV header: node_id\n  --edges PATH          TSV header: source<TAB>target\n  --cascades PATH       TSV header: cascade_id<TAB>started_at<TAB>observation_end_step\n  --activations PATH    TSV header: cascade_id<TAB>node_id<TAB>step\n  --seeds PATH          TSV header: node_id\n  --output PATH         Atomic JSON receipt destination\n\nOptional:\n  --blocked-nodes PATH  One-column node_id TSV\n  --horizon N           Forecast steps (default: 30)\n  --simulations N       Seeded simulations (default: 1000)\n  --seed N              64-bit random seed (default: 42)\n  --transmission-multiplier N  Intervention multiplier (default: 0.75)\n  --max-runtime-ms N    Monotonic deadline, max 600000 (default: 30000)\n  --overwrite           Atomically replace an existing output\n  -h, --help            Show this help\n  --version             Show the package version"
    );
}
