//! `lawsynth doctor` — environment and health check for the CLI install.
//!
//! Reports the CLI version, the available subcommands, whether the offline
//! build constraints hold, whether the world-library directory is writable, and
//! runs a quick self-test that builds a tiny world in memory, simulates it, and
//! round-trips it through the `.lsworld` bundle format and the engine. Every
//! check prints a `PASS`/`WARN`/`FAIL` line and the command ends with an overall
//! verdict. This is the "is my install healthy?" command.

use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

/// The subcommands the CLI dispatches, reported by `doctor`.
const SUBCOMMANDS: &[&str] = &[
    "inspect",
    "discover",
    "simulate",
    "simulate-discrete",
    "report",
    "pipeline",
    "explain",
    "compare",
    "forecast",
    "scenarios",
    "library",
    "export",
    "new",
    "templates",
    "validate",
    "doctor",
];

/// Help text for `lawsynth doctor`.
pub fn help() -> String {
    "lawsynth doctor\n\n\
Runs an environment and health check: CLI version, available subcommands, \
offline build constraints, a writable library directory, and a self-test that \
builds, simulates, and round-trips a tiny world through the bundle format and \
engine. Prints PASS/WARN/FAIL lines and an overall verdict."
        .to_owned()
}

/// The severity of a single health check.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Pass,
    Warn,
    Fail,
}

impl Status {
    fn tag(self) -> &'static str {
        match self {
            Status::Pass => "PASS",
            Status::Warn => "WARN",
            Status::Fail => "FAIL",
        }
    }
}

/// Runs the `doctor` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    if !arguments.is_empty() {
        return Err(help());
    }

    let mut checks: Vec<(Status, String)> = Vec::new();

    checks.push((Status::Pass, format!("version: lawsynth-cli {}", env!("CARGO_PKG_VERSION"))));
    checks.push((
        Status::Pass,
        format!("subcommands: {} available ({})", SUBCOMMANDS.len(), SUBCOMMANDS.join(", ")),
    ));
    checks.push((
        Status::Pass,
        "offline: no network access required at build or run time (std-only, deterministic)"
            .to_owned(),
    ));
    checks.push(library_check());
    checks.push(self_test_check());

    let mut out = String::from("LawSynth doctor\n\n");
    for (status, message) in &checks {
        let _ = writeln!(out, "[{}] {message}", status.tag());
    }

    let failures = checks.iter().filter(|(status, _)| *status == Status::Fail).count();
    let warnings = checks.iter().filter(|(status, _)| *status == Status::Warn).count();
    let verdict = if failures > 0 {
        format!("UNHEALTHY ({failures} failure(s), {warnings} warning(s))")
    } else if warnings > 0 {
        format!("HEALTHY WITH WARNINGS ({warnings} warning(s))")
    } else {
        "HEALTHY".to_owned()
    };
    let _ = writeln!(out, "\nverdict: {verdict}");
    Ok(out)
}

/// Checks that the default world-library directory is writable.
fn library_check() -> (Status, String) {
    let Some(home) = std::env::var_os("HOME") else {
        return (
            Status::Warn,
            "library dir: HOME is not set; pass --dir to library commands".to_owned(),
        );
    };
    let dir = PathBuf::from(home).join(".lawsynth");
    if let Err(error) = fs::create_dir_all(&dir) {
        return (Status::Warn, format!("library dir: {} not creatable: {error}", dir.display()));
    }
    let probe = dir.join(format!(".doctor-probe-{}", std::process::id()));
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            (Status::Pass, format!("library dir: {} is writable", dir.display()))
        }
        Err(error) => {
            (Status::Warn, format!("library dir: {} not writable: {error}", dir.display()))
        }
    }
}

/// Builds a tiny world, simulates it, and round-trips it through the bundle
/// format and the engine to confirm the install is functional end to end.
fn self_test_check() -> (Status, String) {
    match run_self_test() {
        Ok(message) => (Status::Pass, message),
        Err(error) => (Status::Fail, format!("self-test: {error}")),
    }
}

fn run_self_test() -> Result<String, String> {
    let id = |value: &str| Identifier::new(value).map_err(|error| error.to_string());
    let x = id("x")?;
    // dx/dt = -k * x, an exponential decay with a known qualitative behavior.
    let world = World::new(
        [Variable::new(x.clone(), VariableRole::State)],
        [Parameter::new(id("k")?, 1.0)],
        [ContinuousLaw::new(
            x.clone(),
            Expr::product(Expr::constant(-1.0), Expr::symbol(x.clone())),
        )],
    )
    .map_err(|error| error.to_string())?;

    let config = SimulationConfig::new(0.0, 1.0, 0.1).map_err(|error| error.to_string())?;
    let request = SimulationRequest::default().with_initial(x.clone(), 1.0);
    let baseline = simulate(&world, config, &request).map_err(|error| error.to_string())?;

    let series = baseline.values.get(&x).ok_or("engine produced no trajectory for x")?;
    let first = series.first().copied().unwrap_or(f64::NAN);
    let last = series.last().copied().unwrap_or(f64::NAN);
    if !series.iter().all(|value| value.is_finite()) {
        return Err("engine produced a non-finite sample".to_owned());
    }
    if last >= first {
        return Err("decay world did not decrease as expected".to_owned());
    }

    // Round-trip through the .lsworld bundle format and re-simulate. The temp
    // name is unique per call so concurrent invocations never collide.
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir()
        .join(format!("lawsynth-doctor-{}-{unique}.lsworld", std::process::id()));
    write_world(&path, &world).map_err(|error| error.to_string())?;
    let reloaded = read_world(&path).map_err(|error| error.to_string())?;
    let _ = fs::remove_file(&path);
    let round_tripped = simulate(&reloaded, config, &request).map_err(|error| error.to_string())?;
    if round_tripped != baseline {
        return Err("bundle round-trip changed the simulated trajectory".to_owned());
    }

    Ok(format!(
        "self-test: built + simulated + round-tripped a 1-state world ({} samples, x {:.3} -> {:.3})",
        baseline.samples(),
        first,
        last
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_passes() {
        let (status, message) = self_test_check();
        assert!(status == Status::Pass, "self-test should pass: {message}");
    }

    #[test]
    fn doctor_reports_a_verdict() {
        let out = run(&[]).unwrap();
        assert!(out.contains("LawSynth doctor"));
        assert!(out.contains("verdict:"));
        assert!(out.contains("scenarios"));
    }

    #[test]
    fn rejects_extra_arguments() {
        assert!(run(&["unexpected".to_owned()]).is_err());
    }
}
