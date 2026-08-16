use lawsynth_differentiate::weak_derivative_integral;

fn main() {
    let time = [0.0, 1.0, 2.0, 3.0];
    let values = [0.0, 1.0, 4.0, 9.0];
    let weights = [1.0, 1.0, 1.0, 1.0];
    let weight_derivative = [0.0, 0.0, 0.0, 0.0];
    println!(
        "weak derivative integral: {}",
        weak_derivative_integral(&time, &values, &weights, &weight_derivative).unwrap()
    );
}
