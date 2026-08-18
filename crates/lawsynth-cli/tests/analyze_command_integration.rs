//! End-to-end CLI tests for the one-shot `analyze` command, which combines
//! `stability`, `lyapunov`, and `invariants` into a single dynamics report.
//! Each test drives the real engines through `lawsynth_cli::run` on a small
//! deterministic world and asserts both the text and the `--json` shape.

use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::write_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_world::{ContinuousLaw, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn temp_dir(tag: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

/// A damped oscillator x' = y, y' = -x - 0.3y: the origin is a stable spiral and
/// the flow contracts phase-space volume (dissipative), with no polynomial
/// conserved quantity.
fn damped_oscillator_world() -> World {
    World::new(
        [Variable::new(id("x"), VariableRole::State), Variable::new(id("y"), VariableRole::State)],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
            ContinuousLaw::new(
                id("y"),
                Expr::difference(
                    Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x"))),
                    Expr::product(Expr::constant(0.3), Expr::symbol(id("y"))),
                ),
            ),
        ],
    )
    .unwrap()
}

/// A harmonic oscillator x' = y, y' = -x: the origin is a center (marginal,
/// inconclusive), energy x^2 + y^2 is conserved, and the flow is neither chaotic
/// nor dissipative (neutral/conservative).
fn harmonic_oscillator_world() -> World {
    World::new(
        [Variable::new(id("x"), VariableRole::State), Variable::new(id("y"), VariableRole::State)],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
            ContinuousLaw::new(id("y"), Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x")))),
        ],
    )
    .unwrap()
}

fn write_bundle(directory: &Path, name: &str, world: &World) -> String {
    let bundle = directory.join(name);
    write_world(&bundle, world).unwrap();
    bundle.display().to_string()
}

#[test]
fn analyze_damped_oscillator_text_reports_stable_spiral_and_dissipative() {
    let directory = temp_dir("analyze-damped-text");
    let bundle = write_bundle(&directory, "damped.lsworld", &damped_oscillator_world());

    let output = run(&[
        "analyze".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--initial".to_owned(),
        "x=1,y=1".to_owned(),
    ])
    .unwrap();

    // The three labelled parts are present.
    assert!(output.contains("== Stability =="), "output: {output}");
    assert!(output.contains("== Lyapunov spectrum =="), "output: {output}");
    assert!(output.contains("== Invariants =="), "output: {output}");

    // Stability: origin is a stable spiral.
    assert!(output.contains("stable spiral"), "output: {output}");
    // Lyapunov: dissipative verdict.
    assert!(output.contains("verdict: dissipative"), "output: {output}");
    // Invariants: no polynomial conserved quantity for a dissipative system.
    assert!(output.contains("No conserved quantity"), "output: {output}");
    // Consolidated caveats are surfaced.
    assert!(output.contains("Caveats:"), "output: {output}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn analyze_damped_oscillator_json_carries_three_sub_objects() {
    let directory = temp_dir("analyze-damped-json");
    let bundle = write_bundle(&directory, "damped.lsworld", &damped_oscillator_world());

    let json = run(&[
        "analyze".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--initial".to_owned(),
        "x=1,y=1".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();

    // Top-level shape.
    assert!(json.contains("\"world\":"), "json: {json}");
    assert!(json.contains("\"states\": [\"x\", \"y\"]"), "json: {json}");
    assert!(json.contains("\"stability\": {"), "json: {json}");
    assert!(json.contains("\"lyapunov\": {"), "json: {json}");
    assert!(json.contains("\"invariants\": {"), "json: {json}");

    // Stability sub-object keys mirror the standalone command.
    assert!(json.contains("\"seeds_total\":"), "json: {json}");
    assert!(json.contains("\"fixed_points\": ["), "json: {json}");
    assert!(json.contains("\"classification\": \"stable spiral\""), "json: {json}");

    // Lyapunov sub-object keys mirror the standalone command.
    assert!(json.contains("\"exponents\": ["), "json: {json}");
    assert!(json.contains("\"largest\":"), "json: {json}");
    assert!(json.contains("\"sum\":"), "json: {json}");
    assert!(json.contains("\"kaplan_yorke_dimension\":"), "json: {json}");
    assert!(json.contains("\"chaotic\": false"), "json: {json}");

    // Invariants sub-object keys mirror the standalone command.
    assert!(json.contains("\"basis_labels\": ["), "json: {json}");
    assert!(json.contains("\"invariants\": ["), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn analyze_harmonic_oscillator_text_reports_center_energy_and_neutral() {
    let directory = temp_dir("analyze-harmonic-text");
    let bundle = write_bundle(&directory, "harmonic.lsworld", &harmonic_oscillator_world());

    let output = run(&[
        "analyze".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--degree".to_owned(),
        "2".to_owned(),
    ])
    .unwrap();

    // Stability: origin is a center, reported as marginal/inconclusive.
    assert!(output.contains("center"), "output: {output}");
    assert!(output.contains("inconclusive"), "output: {output}");
    // Lyapunov: neutral/conservative verdict.
    assert!(output.contains("verdict: neutral/conservative"), "output: {output}");
    // Invariants: energy x^2 + y^2 is a conserved quantity.
    assert!(output.contains("Conserved quantity"), "output: {output}");
    assert!(output.contains("x^2"), "output: {output}");
    assert!(output.contains("y^2"), "output: {output}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn analyze_harmonic_oscillator_json_reports_energy_and_neutral_spectrum() {
    let directory = temp_dir("analyze-harmonic-json");
    let bundle = write_bundle(&directory, "harmonic.lsworld", &harmonic_oscillator_world());

    let json = run(&[
        "analyze".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--degree".to_owned(),
        "2".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();

    // Stability sub-object: center is inconclusive.
    assert!(
        json.contains("\"classification\": \"center (marginal, inconclusive)\""),
        "json: {json}"
    );
    assert!(json.contains("\"inconclusive\": true"), "json: {json}");
    // Lyapunov sub-object: not chaotic.
    assert!(json.contains("\"chaotic\": false"), "json: {json}");
    // Invariants sub-object: a conserved combination over x^2 and y^2.
    assert!(json.contains("\"combination\":"), "json: {json}");
    assert!(json.contains("x^2"), "json: {json}");
    assert!(json.contains("y^2"), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn analyze_requires_a_search_box() {
    let directory = temp_dir("analyze-no-box");
    let bundle = write_bundle(&directory, "harmonic.lsworld", &harmonic_oscillator_world());

    let error = run(&["analyze".to_owned(), bundle, "--initial".to_owned(), "x=1,y=0".to_owned()])
        .unwrap_err();
    assert!(error.contains("--box is required"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}
