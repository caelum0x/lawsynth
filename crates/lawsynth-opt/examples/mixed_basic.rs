use lawsynth_opt::{CoordinateConfig, ParameterBounds, mixed_minimize};

fn main() {
    let result = mixed_minimize(
        ["linear", "quadratic"],
        &[0.0],
        ParameterBounds::new(-5.0, 5.0).unwrap(),
        CoordinateConfig::default(),
        |model, point| match *model {
            "linear" => (point[0] - 2.0).powi(2),
            _ => (point[0] + 1.0).powi(2),
        },
    )
    .unwrap();
    println!(
        "selected {} with continuous {:?} and objective {}",
        result.discrete, result.continuous, result.objective
    );
}
