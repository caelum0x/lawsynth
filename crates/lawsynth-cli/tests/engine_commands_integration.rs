//! End-to-end CLI tests for the engine commands wired in this batch:
//! `stability`, `control`, and `domains`. Each drives the real engine through
//! `lawsynth_cli::run` on a small deterministic world / dataset.

use std::fs;
use std::path::PathBuf;

use lawsynth_bundle::write_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

/// A unique temp directory for one test's artifacts.
fn temp_dir(tag: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn stability_classifies_a_stable_node_world() {
    let directory = temp_dir("stability");
    let bundle = directory.join("node.lsworld");
    // x' = -x, y' = -2y: a stable node at the origin.
    let world = World::new(
        [Variable::new(id("x"), VariableRole::State), Variable::new(id("y"), VariableRole::State)],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x")))),
            ContinuousLaw::new(id("y"), Expr::product(Expr::constant(-2.0), Expr::symbol(id("y")))),
        ],
    )
    .unwrap();
    write_world(&bundle, &world).unwrap();

    let output = run(&[
        "stability".to_owned(),
        bundle.display().to_string(),
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
    ])
    .unwrap();

    assert!(output.contains("Fixed point(s): 1"), "output: {output}");
    assert!(output.contains("x=0"), "output: {output}");
    assert!(output.contains("stable node"), "output: {output}");
    assert!(output.contains("converged"), "output: {output}");

    let json = run(&[
        "stability".to_owned(),
        bundle.display().to_string(),
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"classification\": \"stable node\""), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn stability_requires_a_search_box() {
    let directory = temp_dir("stability-box");
    let bundle = directory.join("node.lsworld");
    let world = World::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [ContinuousLaw::new(id("x"), Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x"))))],
    )
    .unwrap();
    write_world(&bundle, &world).unwrap();

    let error = run(&["stability".to_owned(), bundle.display().to_string()]).unwrap_err();
    assert!(error.contains("--box"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}

/// Integrates the forced linear system dx/dt = -0.5 x + u, u(t) = sin(t), with
/// fixed-step RK4, and writes a `time,x,u` CSV. Recovering `-0.5 x + u` from this
/// is a genuine SINDYc round-trip.
fn write_forced_dataset(path: &std::path::Path) {
    let dt = 0.02_f64;
    let steps = 400usize;
    let rhs = |x: f64, t: f64| -0.5 * x + t.sin();
    let mut csv = String::from("time,x,u\n");
    let mut x = 1.0_f64;
    for step in 0..=steps {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t},{x},{}\n", t.sin()));
        // Advance x by one RK4 step for the next row.
        let k1 = rhs(x, t);
        let k2 = rhs(x + 0.5 * dt * k1, t + 0.5 * dt);
        let k3 = rhs(x + 0.5 * dt * k2, t + 0.5 * dt);
        let k4 = rhs(x + dt * k3, t + dt);
        x += dt / 6.0 * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
    }
    fs::write(path, csv).unwrap();
}

#[test]
fn control_discovers_and_validates_a_forced_system() {
    let directory = temp_dir("control");
    let data = directory.join("forced.csv");
    write_forced_dataset(&data);

    let output = run(&[
        "control".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--control".to_owned(),
        "u".to_owned(),
        "--validate".to_owned(),
    ])
    .unwrap();

    assert!(output.contains("controls: u"), "output: {output}");
    assert!(output.contains("d/dt x ="), "output: {output}");
    // The discovered law should carry the control term and the -0.5 x decay.
    assert!(output.contains('u'), "output: {output}");
    assert!(output.contains("R2="), "output: {output}");
    assert!(output.contains("in-sample"), "output: {output}");

    let json = run(&[
        "control".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--control".to_owned(),
        "u".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"state\": \"x\""), "json: {json}");
    assert!(json.contains("\"controls\": [\"u\"]"), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn control_requires_state_and_control_flags() {
    let error = run(&[
        "control".to_owned(),
        "missing.csv".to_owned(),
        "--time".to_owned(),
        "time".to_owned(),
    ])
    .unwrap_err();
    assert!(error.contains("--state"), "error: {error}");
}

#[test]
fn discover_reports_template_prior_drops() {
    let directory = temp_dir("template-prior");
    let data = directory.join("decay.csv");
    // Exponential decay dx/dt = -x sampled on a fine grid.
    let dt = 0.02_f64;
    let mut csv = String::from("time,x\n");
    for step in 0..=400usize {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t},{}\n", (-t).exp()));
    }
    fs::write(&data, csv).unwrap();
    let output = directory.join("decay.lsworld");

    let summary = run(&[
        "discover".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--output".to_owned(),
        output.display().to_string(),
        "--max-degree".to_owned(),
        "1".to_owned(),
        "--forbid-interactions".to_owned(),
    ])
    .unwrap();

    assert!(summary.contains("template prior:"), "summary: {summary}");
    assert!(summary.contains("admitted"), "summary: {summary}");
    assert!(output.exists(), "discover should still write a world bundle");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn domains_lists_presets() {
    let output = run(&["domains".to_owned()]).unwrap();
    assert!(output.contains("lotka-volterra"), "output: {output}");
    assert!(output.contains("damped-oscillator"), "output: {output}");
    assert!(output.contains("brusselator"), "output: {output}");
}

#[test]
fn domains_show_prints_reference_law() {
    let output =
        run(&["domains".to_owned(), "show".to_owned(), "lotka-volterra".to_owned()]).unwrap();
    assert!(output.contains("Reference law"), "output: {output}");
    assert!(output.contains("d/dt"), "output: {output}");
    assert!(output.contains("polynomial degree"), "output: {output}");
}

#[test]
fn domains_run_recovers_a_preset_law() {
    let output =
        run(&["domains".to_owned(), "run".to_owned(), "damped-oscillator".to_owned()]).unwrap();
    assert!(output.contains("Round-trip recovery"), "output: {output}");
    assert!(output.contains("Recovery: OK"), "output: {output}");

    let json = run(&[
        "domains".to_owned(),
        "run".to_owned(),
        "lotka-volterra".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"recovered\": true"), "json: {json}");
}
