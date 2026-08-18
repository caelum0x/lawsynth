use lawsynth_core::Identifier;
use lawsynth_expr::{Environment, evaluate, parse, print};

fn main() {
    let expression = parse("2 * x + sin(y)").expect("valid expression");
    let values = Environment::from([
        (Identifier::new("x").unwrap(), 3.0),
        (Identifier::new("y").unwrap(), std::f64::consts::FRAC_PI_2),
    ]);
    println!("{} = {}", print(&expression), evaluate(&expression, &values).unwrap());
}
