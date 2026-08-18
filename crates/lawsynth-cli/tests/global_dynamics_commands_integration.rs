//! End-to-end CLI tests for the global-dynamics and control commands wired in
//! this batch: `lyapunov`, `basins`, `network`, and `mpc`. Each drives the real
//! engine through `lawsynth_cli::run` on a small deterministic world or dataset.

use std::fs;
use std::path::{Path, PathBuf};

use lawsynth_bundle::write_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::{BinaryOperator, Expr, UnaryOperator};
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

fn write_bundle(directory: &Path, name: &str, world: &World) -> String {
    let bundle = directory.join(name);
    write_world(&bundle, world).unwrap();
    bundle.display().to_string()
}

/// Extracts the numeric value that follows `"key":` in a JSON blob.
fn json_number(json: &str, key: &str) -> f64 {
    let needle = format!("\"{key}\":");
    let start = json.find(&needle).unwrap_or_else(|| panic!("missing key {key} in {json}"));
    let tail = json[start + needle.len()..].trim_start();
    let end = tail
        .find(|c: char| !(c.is_ascii_digit() || matches!(c, '-' | '+' | '.' | 'e' | 'E')))
        .unwrap_or(tail.len());
    tail[..end].parse().unwrap_or_else(|_| panic!("bad number for {key}: {tail}"))
}

/// The harmonic oscillator x' = y, y' = -x: a conservative center whose Lyapunov
/// exponents are all zero (no chaos, no dissipation).
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

/// A stable node x' = -x, y' = -2y at the origin: both Lyapunov exponents are
/// strictly negative, so it is unambiguously non-chaotic.
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

/// The bistable 1-D flow x' = x - x^3: stable wells at x = ±1, a saddle at 0.
fn bistable_world() -> World {
    let cube = Expr::binary(BinaryOperator::Power, Expr::symbol(id("x")), Expr::constant(3.0));
    let field = Expr::difference(Expr::symbol(id("x")), cube);
    World::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [ContinuousLaw::new(id("x"), field)],
    )
    .unwrap()
}

/// The double integrator x' = y, y' = u with a control input `u`. Its origin is
/// stabilizable, so successive-linearization LQR-MPC drives it to a setpoint.
fn double_integrator_world() -> World {
    World::new(
        [
            Variable::new(id("x"), VariableRole::State),
            Variable::new(id("y"), VariableRole::State),
            Variable::new(id("u"), VariableRole::Control),
        ],
        [],
        [
            ContinuousLaw::new(id("x"), Expr::symbol(id("y"))),
            ContinuousLaw::new(id("y"), Expr::symbol(id("u"))),
        ],
    )
    .unwrap()
}

/// Integrates the directed linear chain x1' = -x1, x2' = 2 x1 - x2,
/// x3' = 2 x2 - x3 with fixed-step RK4 from (1, 0, 0) and writes a
/// `time,x1,x2,x3` CSV. The couplings x1 -> x2 and x2 -> x3 are the only
/// cross-node influences, so network discovery should recover exactly that chain.
fn write_chain_dataset(path: &Path) {
    let dt = 0.05_f64;
    let steps = 400usize;
    let rhs = |s: [f64; 3]| [-s[0], 2.0 * s[0] - s[1], 2.0 * s[1] - s[2]];
    let add =
        |s: [f64; 3], k: [f64; 3], h: f64| [s[0] + h * k[0], s[1] + h * k[1], s[2] + h * k[2]];
    let mut csv = String::from("time,x1,x2,x3\n");
    let mut s = [1.0_f64, 0.0, 0.0];
    for step in 0..=steps {
        let t = step as f64 * dt;
        csv.push_str(&format!("{t},{},{},{}\n", s[0], s[1], s[2]));
        let k1 = rhs(s);
        let k2 = rhs(add(s, k1, dt / 2.0));
        let k3 = rhs(add(s, k2, dt / 2.0));
        let k4 = rhs(add(s, k3, dt));
        for i in 0..3 {
            s[i] += dt / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
    }
    fs::write(path, csv).unwrap();
}

#[test]
fn lyapunov_reports_near_zero_exponents_for_the_harmonic_oscillator() {
    let directory = temp_dir("lyapunov-osc");
    let bundle = write_bundle(&directory, "osc.lsworld", &harmonic_oscillator_world());

    let output = run(&[
        "lyapunov".to_owned(),
        bundle.clone(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--steps".to_owned(),
        "4000".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("Lyapunov spectrum"), "output: {output}");
    assert!(output.contains("kaplan-yorke dim"), "output: {output}");

    let json = run(&[
        "lyapunov".to_owned(),
        bundle,
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--steps".to_owned(),
        "4000".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    // A conservative center: every exponent (hence the largest) and the sum are
    // numerically zero, and the divergence sum is the tightest quantity.
    assert!(json_number(&json, "largest").abs() < 5e-2, "largest not ~0: {json}");
    assert!(json_number(&json, "sum").abs() < 5e-2, "sum not ~0: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn lyapunov_flags_a_stable_node_as_non_chaotic() {
    let directory = temp_dir("lyapunov-node");
    let bundle = write_bundle(&directory, "node.lsworld", &stable_node_world());

    let output = run(&[
        "lyapunov".to_owned(),
        bundle,
        "--initial".to_owned(),
        "x=1,y=1".to_owned(),
        "--steps".to_owned(),
        "3000".to_owned(),
    ])
    .unwrap();
    // x' = -x, y' = -2y: exponents ≈ -1 and -2, so the largest is strictly
    // negative and the command must not claim chaos.
    assert!(output.contains("not positive"), "output: {output}");
    assert!(output.contains("no chaos"), "output: {output}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn lyapunov_requires_an_initial_condition() {
    let directory = temp_dir("lyapunov-missing");
    let bundle = write_bundle(&directory, "osc.lsworld", &harmonic_oscillator_world());

    let error = run(&["lyapunov".to_owned(), bundle]).unwrap_err();
    assert!(error.contains("--initial"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn basins_finds_two_attractors_for_the_bistable_flow() {
    let directory = temp_dir("basins");
    let bundle = write_bundle(&directory, "bistable.lsworld", &bistable_world());

    let output = run(&[
        "basins".to_owned(),
        bundle.clone(),
        "--box".to_owned(),
        "-2:2".to_owned(),
        "--resolution".to_owned(),
        "11".to_owned(),
        "--tolerance".to_owned(),
        "0.01".to_owned(),
        "--max-time".to_owned(),
        "30".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("Attractor(s): 2"), "output: {output}");
    assert!(output.contains("basin:"), "output: {output}");

    let json = run(&[
        "basins".to_owned(),
        bundle,
        "--box".to_owned(),
        "-2:2".to_owned(),
        "--resolution".to_owned(),
        "11".to_owned(),
        "--tolerance".to_owned(),
        "0.01".to_owned(),
        "--max-time".to_owned(),
        "30".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"attractors\": ["), "json: {json}");
    assert!(json.contains("\"basin_fraction\":"), "json: {json}");
    assert!(json.contains("\"grid_labels\": ["), "json: {json}");
    // The two wells split the settled mass, so at least one attractor cell exists.
    assert!(json.contains("\"a0\""), "json should label some cell for attractor 0: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn basins_requires_a_search_box() {
    let directory = temp_dir("basins-box");
    let bundle = write_bundle(&directory, "bistable.lsworld", &bistable_world());

    let error = run(&["basins".to_owned(), bundle]).unwrap_err();
    assert!(error.contains("--box"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn network_recovers_the_directed_chain() {
    let directory = temp_dir("network");
    let data = directory.join("chain.csv");
    write_chain_dataset(&data);

    let output = run(&[
        "network".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x1,x2,x3".to_owned(),
        "--edge-threshold".to_owned(),
        "0.5".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("nodes:   x1, x2, x3"), "output: {output}");
    assert!(output.contains("x1 -> x2"), "output: {output}");
    assert!(output.contains("x2 -> x3"), "output: {output}");

    let json = run(&[
        "network".to_owned(),
        data.display().to_string(),
        "--time".to_owned(),
        "time".to_owned(),
        "--state".to_owned(),
        "x1,x2,x3".to_owned(),
        "--edge-threshold".to_owned(),
        "0.5".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"nodes\": [\"x1\", \"x2\", \"x3\"]"), "json: {json}");
    assert!(json.contains("\"adjacency\": [["), "json: {json}");
    assert!(json.contains("\"driver\": \"x1\", \"target\": \"x2\""), "json: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn network_requires_at_least_two_states() {
    let directory = temp_dir("network-one");
    let data = directory.join("chain.csv");
    write_chain_dataset(&data);

    let error = run(&[
        "network".to_owned(),
        data.display().to_string(),
        "--state".to_owned(),
        "x1".to_owned(),
    ])
    .unwrap_err();
    assert!(error.contains("at least two nodes"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn mpc_regulates_the_double_integrator_to_the_origin() {
    let directory = temp_dir("mpc");
    let bundle = write_bundle(&directory, "double.lsworld", &double_integrator_world());

    let output = run(&[
        "mpc".to_owned(),
        bundle.clone(),
        "--control".to_owned(),
        "u".to_owned(),
        "--setpoint".to_owned(),
        "x=0,y=0".to_owned(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--dt".to_owned(),
        "0.05".to_owned(),
        "--steps".to_owned(),
        "200".to_owned(),
    ])
    .unwrap();
    assert!(output.contains("Model-predictive control"), "output: {output}");
    assert!(output.contains("final error norm:"), "output: {output}");

    let json = run(&[
        "mpc".to_owned(),
        bundle,
        "--control".to_owned(),
        "u".to_owned(),
        "--setpoint".to_owned(),
        "x=0,y=0".to_owned(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
        "--dt".to_owned(),
        "0.05".to_owned(),
        "--steps".to_owned(),
        "200".to_owned(),
        "--json".to_owned(),
    ])
    .unwrap();
    assert!(json.contains("\"state_trajectory\": [["), "json: {json}");
    assert!(json.contains("\"control_trajectory\": [["), "json: {json}");
    // The controller drives the state close to the origin.
    assert!(json_number(&json, "final_error_norm") < 1e-2, "did not converge: {json}");

    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn mpc_requires_control_setpoint_and_initial() {
    let directory = temp_dir("mpc-missing");
    let bundle = write_bundle(&directory, "double.lsworld", &double_integrator_world());

    let error = run(&[
        "mpc".to_owned(),
        bundle,
        "--setpoint".to_owned(),
        "x=0,y=0".to_owned(),
        "--initial".to_owned(),
        "x=1,y=0".to_owned(),
    ])
    .unwrap_err();
    assert!(error.contains("--control"), "error: {error}");

    fs::remove_dir_all(directory).unwrap();
}
