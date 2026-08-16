use lawsynth_sim::Trajectory;
use std::fmt::Write;
/// Formats a trajectory as stable RFC-4180-compatible numeric CSV.
pub fn trajectory_csv(trajectory: &Trajectory) -> String {
    let mut output = String::from("time");
    for id in trajectory.values.keys() {
        write!(&mut output, ",{id}").expect("string write");
    }
    output.push('\n');
    for row in 0..trajectory.samples() {
        write!(&mut output, "{:.17e}", trajectory.time[row]).expect("string write");
        for values in trajectory.values.values() {
            write!(&mut output, ",{:.17e}", values[row]).expect("string write");
        }
        output.push('\n');
    }
    output
}
