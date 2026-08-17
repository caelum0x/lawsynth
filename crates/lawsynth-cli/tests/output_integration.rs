use std::fs;

use lawsynth_bundle::write_discrete_world;
use lawsynth_cli::run;
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{DiscreteLaw, DiscreteWorld, Variable, VariableRole};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

#[test]
fn discrete_command_simulates_a_serialized_world() {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-discrete-{}-{}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    let bundle = directory.join("recurrence.lsworld");
    let world = DiscreteWorld::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [DiscreteLaw::new(id("x"), Expr::sum(Expr::symbol(id("x")), Expr::constant(1.0)))],
    )
    .unwrap();
    write_discrete_world(&bundle, &world).unwrap();

    let output = run(&[
        "simulate-discrete".to_owned(),
        bundle.display().to_string(),
        "--initial".to_owned(),
        "x=2".to_owned(),
        "--steps".to_owned(),
        "3".to_owned(),
    ])
    .unwrap();

    assert_eq!(output.lines().last().unwrap(), "3.00000000000000000e0,5.00000000000000000e0");
    fs::remove_dir_all(directory).unwrap();
}
