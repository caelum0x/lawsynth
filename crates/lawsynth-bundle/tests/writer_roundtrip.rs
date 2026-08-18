use std::fs;

use lawsynth_bundle::{read_discrete_world, read_world, write_discrete_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_units::Unit;
use lawsynth_world::{
    ContinuousLaw, DiscreteLaw, DiscreteWorld, Parameter, Variable, VariableRole, World,
};

fn id(value: &str) -> Identifier {
    Identifier::new(value).unwrap()
}

fn test_world() -> World {
    World::new(
        [
            Variable::new(id("inventory"), VariableRole::State).with_unit(Unit::dimensionless()),
            Variable::new(id("promotion"), VariableRole::Control).with_unit(Unit::dimensionless()),
        ],
        [Parameter::new(id("adjustment_rate"), 0.5).with_unit(Unit::parse("s^-1").unwrap())],
        [ContinuousLaw::new(
            id("inventory"),
            Expr::product(
                Expr::symbol(id("adjustment_rate")),
                Expr::difference(Expr::symbol(id("promotion")), Expr::symbol(id("inventory"))),
            ),
        )],
    )
    .unwrap()
}

#[test]
fn bundle_round_trips_byte_stably() {
    let directory = std::env::temp_dir().join(format!("lawsynth-bundle-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.lsworld");
    let second = directory.join("second.lsworld");

    write_world(&first, &test_world()).unwrap();
    let loaded = read_world(&first).unwrap();
    write_world(&second, &loaded).unwrap();

    assert_eq!(loaded, test_world());
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn discrete_bundle_round_trips_byte_stably() {
    let world = DiscreteWorld::new(
        [Variable::new(id("x"), VariableRole::State)],
        [],
        [DiscreteLaw::new(id("x"), Expr::sum(Expr::symbol(id("x")), Expr::constant(1.0)))],
    )
    .unwrap();
    let directory = std::env::temp_dir().join(format!("lawsynth-discrete-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.lsworld");
    let second = directory.join("second.lsworld");

    write_discrete_world(&first, &world).unwrap();
    let loaded = read_discrete_world(&first).unwrap();
    write_discrete_world(&second, &loaded).unwrap();

    assert_eq!(loaded, world);
    assert_eq!(fs::read(&first).unwrap(), fs::read(&second).unwrap());
    assert!(read_world(&first).is_err());
    fs::remove_dir_all(directory).unwrap();
}

#[test]
fn five_reference_worlds_match_after_bundle_round_trip() {
    let id = |value| Identifier::new(value).unwrap();
    let worlds = vec![
        ("constant", Expr::constant(1.0)),
        ("growth", Expr::symbol(id("x"))),
        ("decay", Expr::unary(lawsynth_expr::UnaryOperator::Negate, Expr::symbol(id("x")))),
        ("quadratic", Expr::product(Expr::symbol(id("x")), Expr::symbol(id("x")))),
        ("trigonometric", Expr::unary(lawsynth_expr::UnaryOperator::Sin, Expr::symbol(id("x")))),
        ("controlled", Expr::sum(Expr::symbol(id("x")), Expr::symbol(id("u")))),
    ];
    let directory =
        std::env::temp_dir().join(format!("lawsynth-conformance-{}", std::process::id()));
    fs::create_dir_all(&directory).unwrap();
    for (name, expression) in worlds {
        let mut variables = vec![Variable::new(id("x"), VariableRole::State)];
        if name == "controlled" {
            variables.push(Variable::new(id("u"), VariableRole::Control));
        }
        let world = World::new(variables, [], [ContinuousLaw::new(id("x"), expression)]).unwrap();
        let request = if name == "controlled" {
            SimulationRequest::default().with_initial(id("x"), 0.5).with_input(id("u"), 0.25)
        } else {
            SimulationRequest::default().with_initial(id("x"), 0.5)
        };
        let direct =
            simulate(&world, SimulationConfig::new(0.0, 0.1, 0.01).unwrap(), &request).unwrap();
        let path = directory.join(format!("{name}.lsworld"));
        write_world(&path, &world).unwrap();
        let loaded = read_world(&path).unwrap();
        let round_tripped =
            simulate(&loaded, SimulationConfig::new(0.0, 0.1, 0.01).unwrap(), &request).unwrap();
        assert_eq!(direct, round_tripped, "{name}");
    }
}
