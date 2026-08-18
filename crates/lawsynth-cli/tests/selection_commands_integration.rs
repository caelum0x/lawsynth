//! End-to-end CLI tests for the commands wired in this batch: `invariants`
//! (conserved-quantity detection on a world) and `select` (cross-validated
//! hyperparameter selection on a dataset). Each drives the real engine through
//! `lawsynth_cli::run` on a small deterministic world / dataset.

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

/// The undamped harmonic oscillator x' = y, y' = -x. Its energy x² + y² is
/// conserved, so `invariants` should recover a degree-2 quantity with equal
/// weight on x^2 and y^2.
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

/// A stable node x' = -x, y' = -2y: every trajectory decays to the origin, so no
/// nonconstant polynomial quantity is conserved. Exercises the honest empty path.
fn stable_node_world() -> World {
    World::new(
        [Variable::new(id("x"), VariableRole::State), Variable::new(id("y"), VariableRole::State)],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::unary(UnaryOperator::Negate, Expr::symbol(id("x")))),
            ContinuousLaw::new(id("y"), Expr::product(Expr::constant(-2.0), Expr::symbol(id("y")))),
        ],
    )
    .unwrap()
}

fn write_bundle(directory: &Path, name: &str, world: &World) -> String {
    let bundle = directory.join(name);
    write_world(&bundle, world).unwrap();
    bundle.display().to_string()
}

/// Writes exponential decay x(t) = e^{-t} sampled on a fine grid to `time,x` CSV.
/// The underlying law x' = -x is degree-1, deterministically discoverable, and
/// long enough to split into cross-validation folds.
fn write_decay_dataset(path: &Path) {
    let dt = 0.05_f64;
    let steps = 200usize;
    let mut csv = String::from("time,x\n");
    for step in 0..=steps {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t},{}\n", (-t).exp()));
    }
    fs::write(path, csv).unwrap();
}

#[test]
fn invariants_recovers_the_energy_of_a_harmonic_oscillator() {
    let directory = temp_dir("invariants");
    let bundle = write_bundle(&directory, "oscillator.lsworld", &harmonic_oscillator_world());

    let output =
        run(&["invariants".to_owned(), bundle.clone(), "--degree".to_owned(), "2".to_owned()])
            .unwrap();

    assert!(output.contains("Conserved quantity(ies): 1"), "output: {output}");
    // Energy x² + y²: equal-weight combination of the two squares.
    assert!(output.contains("1.00\u{b7}x^2"), "output: {output}");
    assert!(output.contains("1.00\u{b7}y^2"), "output: {output}");
    assert!(output.contains("residual:"), "output: {output}");
    assert!(output.contains("singular value:"), "output: {output}");

    let json = run(&[
        "invariants".to_owned(),
        bundle,
        "--degree".to_owned(),
        "2".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"basis_labels\": ["), "json: {json}");
    assert!(json.contains("\"singular_value\":"), "json: {json}");
    assert!(json.contains("\"combination\":"), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn invariants_is_honest_when_no_conserved_quantity_exists() {
    let directory = temp_dir("invariants-none");
    let bundle = write_bundle(&directory, "node.lsworld", &stable_node_world());

    let output =
        run(&["invariants".to_owned(), bundle, "--degree".to_owned(), "2".to_owned()]).unwrap();

    assert!(
        output.contains("No conserved quantity expressible in the degree-2 library"),
        "output: {output}"
    );

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn select_chooses_a_winner_and_populates_the_audit_table() {
    let directory = temp_dir("select");
    let data = directory.join("decay.csv");
    write_decay_dataset(&data);

    let output = run(&[
        "select".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--degrees".to_owned(),
        "1,2".to_owned(),
        "--thresholds".to_owned(),
        "0.05".to_owned(),
        "--folds".to_owned(),
        "2".to_owned(),
    ])
    .unwrap();

    assert!(output.contains("Model selection"), "output: {output}");
    assert!(output.contains("mean_score"), "output: {output}");
    // The winner is marked in the audit table and summarised.
    assert!(output.contains("<=="), "output: {output}");
    assert!(output.contains("Selected: degree="), "output: {output}");

    let json = run(&[
        "select".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x".to_owned(),
        "--degrees".to_owned(),
        "1,2".to_owned(),
        "--folds".to_owned(),
        "2".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"best_index\":"), "json: {json}");
    assert!(json.contains("\"candidates\": ["), "json: {json}");
    assert!(json.contains("\"is_best\": true"), "json: {json}");
    assert!(json.contains("\"fold_scores\": ["), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn select_requires_state_and_degrees() {
    let directory = temp_dir("select-args");
    let data = directory.join("decay.csv");
    write_decay_dataset(&data);

    let error = run(&[
        "select".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--degrees".to_owned(),
        "1".to_owned(),
    ])
    .unwrap_err();
    assert!(error.contains("--state"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}
