use lawsynth_data::TimeAxis;
use lawsynth_dynamics::central_derivative;
fn main() {
    let time = TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0]).unwrap();
    println!("central derivative: {:?}", central_derivative(&time, &[0.0, 1.0, 4.0, 9.0]).unwrap());
}
