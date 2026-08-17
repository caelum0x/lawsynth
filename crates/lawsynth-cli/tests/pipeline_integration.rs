//! End-to-end smoke tests for the `pipeline`, `report --data`, and
//! `compare --html` product features.

use std::fs;
use std::path::PathBuf;

/// Returns a unique scratch directory under the OS temp dir.
fn scratch(tag: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("lawsynth-cli-{tag}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[&str]) -> Result<String, String> {
    lawsynth_cli::run(&args.iter().map(|value| value.to_string()).collect::<Vec<_>>())
}

#[test]
fn pipeline_runs_end_to_end_and_writes_artifacts() {
    let dir = scratch("pipeline");
    let obs = dir.join("obs.csv");
    let world = dir.join("model.lsworld");
    let report = dir.join("model.report.html");
    let python = dir.join("model.py");
    let config = dir.join("pipeline.toml");

    // Generate a deterministic dataset from a template.
    run(&[
        "new",
        "lotka-volterra",
        "--output",
        dir.join("truth.lsworld").to_str().unwrap(),
        "--data",
        obs.to_str().unwrap(),
        "--samples",
        "200",
    ])
    .unwrap();

    fs::write(
        &config,
        format!(
            "[input]\ncsv = {csv:?}\ntime = \"time\"\nstate = [\"x\", \"y\"]\n\
             [discovery]\ndegree = 2\nthreshold = 0.05\npareto = true\n\
             [validate]\nholdout = 0.2\n\
             [outputs]\nworld = {world:?}\nreport = {report:?}\nexport_python = {python:?}\n",
            csv = obs.to_str().unwrap(),
            world = world.to_str().unwrap(),
            report = report.to_str().unwrap(),
            python = python.to_str().unwrap(),
        ),
    )
    .unwrap();

    let summary = run(&["pipeline", config.to_str().unwrap()]).unwrap();
    assert!(summary.contains("discovered 2 state law(s)"), "summary: {summary}");
    assert!(summary.contains("validate:"), "summary: {summary}");
    assert!(summary.contains("pareto frontier"), "summary: {summary}");

    assert!(world.exists(), "world bundle written");
    assert!(python.exists(), "python export written");
    let html = fs::read_to_string(&report).unwrap();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("Fit vs observations"), "residual overlay section present");
    assert!(!html.contains("<script"), "report is self-contained");

    // Reproducibility: the same config yields the same summary.
    let again = run(&["pipeline", config.to_str().unwrap()]).unwrap();
    assert_eq!(summary, again);
}

#[test]
fn pipeline_example_prints_a_documented_config() {
    let example = run(&["pipeline", "--example"]).unwrap();
    assert!(example.contains("[input]"));
    assert!(example.contains("[discovery]"));
    assert!(example.contains("[outputs]"));
    assert!(example.contains("state = [\"x\", \"y\"]"));
}

#[test]
fn compare_writes_side_by_side_html() {
    let dir = scratch("compare");
    let a = dir.join("a.lsworld");
    let b = dir.join("b.lsworld");
    let html = dir.join("diff.html");
    run(&["new", "lotka-volterra", "--output", a.to_str().unwrap()]).unwrap();
    run(&["new", "sir", "--output", b.to_str().unwrap()]).unwrap();

    let summary = run(&[
        "compare",
        a.to_str().unwrap(),
        b.to_str().unwrap(),
        "--html",
        html.to_str().unwrap(),
    ])
    .unwrap();
    assert!(summary.contains("wrote comparison"), "summary: {summary}");
    let document = fs::read_to_string(&html).unwrap();
    assert!(document.contains("World comparison"));
    assert!(document.contains("Complexity"));
    assert!(!document.contains("<script"));
}
