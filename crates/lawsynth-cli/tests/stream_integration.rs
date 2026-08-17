//! End-to-end coverage for `lawsynth stream`: a regime-switching series must
//! produce a second model with a change record naming the differing terms, and
//! replaying the identical bytes must be byte-for-byte identical.

use std::fs;
use std::path::PathBuf;

use lawsynth_cli::run;

/// RK4 step of a 2-D linear system `f(x, y) -> (x', y')`.
fn rk4(x: f64, y: f64, f: &dyn Fn(f64, f64) -> (f64, f64), dt: f64) -> (f64, f64) {
    let k1 = f(x, y);
    let k2 = f(x + 0.5 * dt * k1.0, y + 0.5 * dt * k1.1);
    let k3 = f(x + 0.5 * dt * k2.0, y + 0.5 * dt * k2.1);
    let k4 = f(x + dt * k3.0, y + dt * k3.1);
    (
        x + dt / 6.0 * (k1.0 + 2.0 * k2.0 + 2.0 * k3.0 + k4.0),
        y + dt / 6.0 * (k1.1 + 2.0 * k2.1 + 2.0 * k3.1 + k4.1),
    )
}

/// Writes a CSV that runs a coupled spiral for the first half and a decoupled
/// decay for the second, switching at the midpoint.
fn write_regime_switch_csv(path: &PathBuf) {
    let dt = 0.02;
    let half = 400;
    let regime_a = |x: f64, y: f64| (-0.5 * x + 2.0 * y, -2.0 * x - 0.5 * y);
    let regime_b = |x: f64, y: f64| (-1.5 * x, -0.3 * y);
    let mut text = String::from("time,x,y\n");
    let mut t = 0.0;
    let (mut x, mut y) = (1.0, 0.5);
    for _ in 0..half {
        text.push_str(&format!("{t:.6},{x:.10},{y:.10}\n"));
        (x, y) = rk4(x, y, &regime_a, dt);
        t += dt;
    }
    // Reseed to a clean state so regime B is excited rather than already decayed.
    (x, y) = (1.0, 1.0);
    for _ in 0..half {
        text.push_str(&format!("{t:.6},{x:.10},{y:.10}\n"));
        (x, y) = rk4(x, y, &regime_b, dt);
        t += dt;
    }
    fs::write(path, text).unwrap();
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "lawsynth-stream-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn stream_detects_a_regime_switch_and_records_the_differing_terms() {
    let dir = temp_dir("detect");
    let csv = dir.join("regime.csv");
    write_regime_switch_csv(&csv);
    let history = dir.join("history.jsonl");

    let summary = run(&[
        "stream".to_owned(),
        csv.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x,y".to_owned(),
        "--window".to_owned(),
        "80".to_owned(),
        "--step".to_owned(),
        "40".to_owned(),
        "--threshold".to_owned(),
        "4".to_owned(),
        "--sustain".to_owned(),
        "2".to_owned(),
        "--output".to_owned(),
        history.display().to_string(),
    ])
    .unwrap();

    // The summary reports exactly one re-discovery (two models total).
    assert!(summary.contains("models produced: 2"), "summary was: {summary}");
    assert!(summary.contains("change points:"), "summary was: {summary}");

    let jsonl = fs::read_to_string(&history).unwrap();
    let lines: Vec<&str> = jsonl.lines().collect();
    assert_eq!(lines.len(), 2, "expected an initial record and one update");

    // The first is the seed model (no prior); the second is a triggered update.
    assert!(lines[0].contains("\"kind\":\"initial\""));
    assert!(lines[0].contains("\"prior_revision\":null"));
    let update = lines[1];
    assert!(update.contains("\"kind\":\"update\""));
    // The update references the seed's revision as its prior.
    assert!(
        update.contains("\"prior_revision\":\"") && !update.contains("\"prior_revision\":null")
    );
    // Its diff must name the terms that differ between the two regimes: the
    // cross-coupling terms (x depends on y, y depends on x) are removed.
    assert!(update.contains("\"diff\":["), "update had no diff: {update}");
    assert!(update.contains("\"kind\":\"removed\""), "expected a removed term: {update}");
    assert!(
        update.contains("\"sustained_windows\":2"),
        "trigger should record the sustained streak: {update}"
    );

    fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn stream_replay_is_byte_for_byte_identical() {
    let dir = temp_dir("replay");
    let csv = dir.join("regime.csv");
    write_regime_switch_csv(&csv);
    let first = dir.join("first.jsonl");
    let second = dir.join("second.jsonl");

    let args = |output: &str| {
        vec![
            "stream".to_owned(),
            csv.display().to_string(),
            "--time".to_owned(),
            "time".to_owned(),
            "--state".to_owned(),
            "x,y".to_owned(),
            "--window".to_owned(),
            "80".to_owned(),
            "--step".to_owned(),
            "40".to_owned(),
            "--output".to_owned(),
            output.to_owned(),
        ]
    };

    let summary_one = run(&args(&first.display().to_string())).unwrap();
    let summary_two = run(&args(&second.display().to_string())).unwrap();

    // The change-record streams must be identical byte-for-byte across replays.
    let stream_one = fs::read(&first).unwrap();
    let stream_two = fs::read(&second).unwrap();
    assert_eq!(stream_one, stream_two, "replayed change-record streams differ");

    // Summaries differ only in the output path line; normalize it away.
    let normalize = |summary: String, output: &str| summary.replace(output, "<HISTORY>");
    assert_eq!(
        normalize(summary_one, &first.display().to_string()),
        normalize(summary_two, &second.display().to_string()),
    );

    fs::remove_dir_all(&dir).unwrap();
}
