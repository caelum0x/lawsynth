//! End-to-end CLI tests for the analysis-layer commands wired in this batch:
//! `bifurcation`, `sensitivity`, `estimate`, and `reduce`. Each drives the real
//! engine through `lawsynth_cli::run` on a small deterministic world.

use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::write_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

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

/// The transcritical normal form x' = mu*x - x^2, with `mu` a declared parameter
/// (value 1.0) that the field references, so it is a genuine free parameter.
fn transcritical_world() -> World {
    let field = Expr::difference(
        Expr::product(Expr::symbol(id("mu")), Expr::symbol(id("x"))),
        Expr::binary(BinaryOperator::Power, Expr::symbol(id("x")), Expr::constant(2.0)),
    );
    World::new(
        [Variable::new(id("x"), VariableRole::State)],
        [Parameter::new(id("mu"), 1.0)],
        [ContinuousLaw::new(id("x"), field)],
    )
    .unwrap()
}

/// A stable node x' = -x, y' = -2y at the origin (autonomous, no parameters).
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

/// A damped oscillator x' = y, y' = -x - 0.3y: a coupled linear system whose
/// origin is a stable spiral. Measuring only `x` is observable (the two states
/// are coupled), so it exercises observer/Kalman design honestly.
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

/// Linear decay x' = -theta * x with `theta` a declared parameter (value 0.5).
fn decay_world() -> World {
    let field = Expr::unary(
        UnaryOperator::Negate,
        Expr::product(Expr::symbol(id("theta")), Expr::symbol(id("x"))),
    );
    World::new(
        [Variable::new(id("x"), VariableRole::State)],
        [Parameter::new(id("theta"), 0.5)],
        [ContinuousLaw::new(id("x"), field)],
    )
    .unwrap()
}

fn write_bundle(directory: &Path, name: &str, world: &World) -> String {
    let bundle = directory.join(name);
    write_world(&bundle, world).unwrap();
    bundle.display().to_string()
}

#[test]
fn bifurcation_detects_a_fold_in_the_transcritical_normal_form() {
    let directory = temp_dir("bifurcation");
    let bundle = write_bundle(&directory, "transcritical.lsworld", &transcritical_world());

    let output = run(&[
        "bifurcation".to_owned(),
        bundle.clone(),
        "--parameter".to_owned(),
        "mu".to_owned(),
        "--range".to_owned(),
        "-1:1".to_owned(),
        "--box".to_owned(),
        "-2:2".to_owned(),
        "--steps".to_owned(),
        "21".to_owned(),
    ])
    .unwrap();

    assert!(output.contains("Detected bifurcation"), "output: {output}");
    assert!(output.contains("fold"), "output: {output}");
    assert!(output.contains("mu* ="), "output: {output}");

    let json = run(&[
        "bifurcation".to_owned(),
        bundle,
        "--parameter".to_owned(),
        "mu".to_owned(),
        "--range".to_owned(),
        "-1:1".to_owned(),
        "--box".to_owned(),
        "-2:2".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"kind\": \"fold\""), "json: {json}");
    assert!(json.contains("\"parameter\": \"mu\""), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn bifurcation_rejects_a_parameter_absent_from_the_laws() {
    let directory = temp_dir("bifurcation-absent");
    // The stable-node world has no free parameter to sweep.
    let bundle = write_bundle(&directory, "node.lsworld", &stable_node_world());

    let error = run(&[
        "bifurcation".to_owned(),
        bundle,
        "--parameter".to_owned(),
        "k".to_owned(),
        "--range".to_owned(),
        "-1:1".to_owned(),
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
    ])
    .unwrap_err();
    assert!(error.contains("does not appear"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn sensitivity_reports_a_negative_final_time_partial_for_decay() {
    let directory = temp_dir("sensitivity");
    let bundle = write_bundle(&directory, "decay.lsworld", &decay_world());

    // x' = -theta*x, x0 = 2, theta = 0.5: dx/dtheta = -t*x0*e^{-theta t} < 0.
    let output = run(&[
        "sensitivity".to_owned(),
        bundle.clone(),
        "--parameters".to_owned(),
        "theta".to_owned(),
        "--initial".to_owned(),
        "x=2".to_owned(),
        "--dt".to_owned(),
        "0.01".to_owned(),
        "--steps".to_owned(),
        "100".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("d x / d theta ="), "output: {output}");
    assert!(output.contains("d x / d theta = -"), "expected negative sensitivity: {output}");

    let json = run(&[
        "sensitivity".to_owned(),
        bundle,
        "--parameters".to_owned(),
        "theta".to_owned(),
        "--initial".to_owned(),
        "x=2".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"parameter\": \"theta\""), "json: {json}");
    assert!(json.contains("\"value\": -"), "json should carry a negative value: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn estimate_designs_a_convergent_observer_by_pole_placement() {
    let directory = temp_dir("estimate");
    let bundle = write_bundle(&directory, "osc.lsworld", &damped_oscillator_world());

    // Measure only x; place both error poles at -3 and -4 (stable => convergent).
    let output = run(&[
        "estimate".to_owned(),
        bundle.clone(),
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--measure".to_owned(),
        "x".to_owned(),
        "--poles".to_owned(),
        "-3,-4".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("Observer gain L"), "output: {output}");
    assert!(output.contains("pole placement"), "output: {output}");
    assert!(output.contains("convergent: yes"), "output: {output}");

    let json = run(&[
        "estimate".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--measure".to_owned(),
        "x".to_owned(),
        "--kalman".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("Observer gain L") || json.contains("\"method\""), "json: {json}");
    assert!(json.contains("Kalman") || json.contains("covariance"), "kalman output: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn estimate_kalman_json_reports_a_gain_and_covariance() {
    let directory = temp_dir("estimate-kalman");
    let bundle = write_bundle(&directory, "osc.lsworld", &damped_oscillator_world());

    let json = run(&[
        "estimate".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--measure".to_owned(),
        "x".to_owned(),
        "--kalman".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"method\": \"kalman\""), "json: {json}");
    assert!(json.contains("\"gain\":"), "json: {json}");
    assert!(json.contains("\"convergent\": true"), "json: {json}");
    assert!(json.contains("\"covariance\": [["), "kalman should report P: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reduce_reports_hankel_singular_values_and_a_reduced_order() {
    let directory = temp_dir("reduce");
    let bundle = write_bundle(&directory, "node.lsworld", &stable_node_world());

    let output = run(&[
        "reduce".to_owned(),
        bundle.clone(),
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--order".to_owned(),
        "1".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("Hankel singular values"), "output: {output}");
    assert!(output.contains("sigma_1 ="), "output: {output}");
    assert!(output.contains("Reduced order: 1 of 2"), "output: {output}");

    let json = run(&[
        "reduce".to_owned(),
        bundle,
        "--box".to_owned(),
        "-1:1,-1:1".to_owned(),
        "--order".to_owned(),
        "1".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"hankel_singular_values\": ["), "json: {json}");
    assert!(json.contains("\"order\": 1"), "json: {json}");
    assert!(json.contains("\"reduced\": {"), "json should carry reduced matrices: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn reduce_requires_an_order_or_tolerance() {
    let directory = temp_dir("reduce-spec");
    let bundle = write_bundle(&directory, "node.lsworld", &stable_node_world());

    let error = run(&["reduce".to_owned(), bundle, "--box".to_owned(), "-1:1,-1:1".to_owned()])
        .unwrap_err();
    assert!(error.contains("--order") || error.contains("--tolerance"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}
