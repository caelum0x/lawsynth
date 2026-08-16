use std::fs;

use lawsynth_bundle::{write_discrete_world, write_world};
use lawsynth_cli::{run, world_summary};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_world::{
    ContinuousLaw, DiscreteLaw, DiscreteWorld, Parameter, Variable, VariableRole, World,
};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn temporary_directory(label: &str) -> std::path::PathBuf {
    let directory = std::env::temp_dir().join(format!(
        "lawsynth-cli-inspect-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&directory).unwrap();
    directory
}

#[test]
fn inspect_reports_a_serialized_continuous_world() {
    let directory = temporary_directory("continuous");
    let bundle = directory.join("growth.lsworld");
    let world = World::new(
        [
            Variable::new(id("x"), VariableRole::State),
            Variable::new(id("u"), VariableRole::Control),
        ],
        [Parameter::new(id("rate"), 2.0)],
        [ContinuousLaw::new(
            id("x"),
            Expr::product(Expr::symbol(id("rate")), Expr::symbol(id("x"))),
        )],
    )
    .unwrap();
    write_world(&bundle, &world).unwrap();

    let output = run(&["inspect".to_owned(), bundle.display().to_string()]).unwrap();

    assert_eq!(
        output,
        "continuous world: 1 states, 2 variables, 1 parameters\n"
    );
    assert_eq!(
        output,
        world_summary("continuous", world.state_ids().count(), 2, 1)
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn inspect_falls_back_to_a_serialized_discrete_world() {
    let directory = temporary_directory("discrete");
    let bundle = directory.join("recurrence.lsworld");
    let world = DiscreteWorld::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [DiscreteLaw::new(
            id("x"),
            Expr::sum(Expr::symbol(id("x")), Expr::constant(1.0)),
        )],
    )
    .unwrap();
    write_discrete_world(&bundle, &world).unwrap();

    let output = run(&["inspect".to_owned(), bundle.display().to_string()]).unwrap();

    assert_eq!(
        output,
        "discrete world: 1 states, 1 variables, 0 parameters\n"
    );
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn inspect_rejects_a_missing_bundle() {
    let error = run(&["inspect".to_owned(), "does-not-exist.lsworld".to_owned()]).unwrap_err();

    assert!(error.contains("No such file") || error.contains("os error"));
}
