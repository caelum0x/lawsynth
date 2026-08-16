use std::fs;

use lawsynth_bundle::write_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn controlled_world() -> World {
    World::new(
        [
            Variable::new(id("x"), VariableRole::State),
            Variable::new(id("u"), VariableRole::Control),
        ],
        [Parameter::new(id("rate"), 1.0)],
        [ContinuousLaw::new(
            id("x"),
            Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("u"))),
        )],
    )
    .unwrap()
}

#[test]
fn simulate_applies_state_parameter_and_input_overrides() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let bundle = directory.join("controlled.lsworld");
    write_world(&bundle, &controlled_world()).unwrap();

    let output = run(&[
        "simulate".to_owned(),
        bundle.display().to_string(),
        "--initial".to_owned(),
        "x=5".to_owned(),
        "--parameter".to_owned(),
        "rate=3".to_owned(),
        "--input".to_owned(),
        "u=2".to_owned(),
        "--start".to_owned(),
        "0".to_owned(),
        "--end".to_owned(),
        "0.1".to_owned(),
        "--step".to_owned(),
        "0.1".to_owned(),
    ])
    .unwrap();

    let final_x: f64 = output
        .lines()
        .nth(2)
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    assert!((final_x - 5.6).abs() < 1e-12);
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn simulate_rejects_invalid_assignments() {
    let error = run(&[
        "simulate".to_owned(),
        "missing.lsworld".to_owned(),
        "--initial".to_owned(),
        "x".to_owned(),
    ])
    .unwrap_err();
    assert_eq!(error, "expected NAME=VALUE");
}

#[test]
fn simulate_applies_a_scheduled_parameter_change() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-scheduled-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let bundle = directory.join("controlled.lsworld");
    write_world(&bundle, &controlled_world()).unwrap();
    let output = run(&[
        "simulate".to_owned(),
        bundle.display().to_string(),
        "--initial".to_owned(),
        "x=0".to_owned(),
        "--input".to_owned(),
        "u=1".to_owned(),
        "--parameter-at".to_owned(),
        "0.5:rate=3".to_owned(),
        "--start".to_owned(),
        "0".to_owned(),
        "--end".to_owned(),
        "1".to_owned(),
        "--step".to_owned(),
        "1".to_owned(),
    ])
    .unwrap();
    let final_x: f64 = output
        .lines()
        .last()
        .unwrap()
        .split(',')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    assert!((final_x - 2.0).abs() < 1e-12);
    fs::remove_dir_all(directory).unwrap();
}
