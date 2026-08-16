use lawsynth_artifact_service::{ArtifactConfig, ArtifactService};

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("lawsynth-artifact: {error}");
        std::process::exit(2);
    }
}

fn run(arguments: Vec<String>) -> Result<(), String> {
    let Some(command) = arguments.first().map(String::as_str) else {
        return Err(usage());
    };
    match command {
        "health" if arguments.len() == 2 => {
            let service = ArtifactService::open(ArtifactConfig::new(&arguments[1]))
                .map_err(|error| error.to_string())?;
            let report = service.health().map_err(|error| error.to_string())?;
            println!(
                "healthy root={} artifacts={} data_bytes={} capacity_bytes={}",
                service.root().display(),
                report.artifact_count,
                report.stored_data_bytes,
                report.capacity_bytes
            );
            Ok(())
        }
        "gc" if arguments.len() == 3 || arguments.len() == 4 => {
            let now = arguments[2]
                .parse::<u64>()
                .map_err(|_| "gc timestamp must be an unsigned Unix timestamp".to_owned())?;
            let dry_run = arguments.get(3).is_some_and(|value| value == "--dry-run");
            if arguments.len() == 4 && !dry_run {
                return Err(usage());
            }
            let service = ArtifactService::open(ArtifactConfig::new(&arguments[1]))
                .map_err(|error| error.to_string())?;
            let report =
                service.collect_garbage(now, dry_run).map_err(|error| error.to_string())?;
            println!("examined={} deleted={}", report.examined, report.deleted.len());
            for id in report.deleted {
                println!("{id}");
            }
            Ok(())
        }
        _ => Err(usage()),
    }
}

fn usage() -> String {
    "usage: lawsynth-artifact health <root> | lawsynth-artifact gc <root> <unix-seconds> [--dry-run]; HTTP serving is not implemented".into()
}
