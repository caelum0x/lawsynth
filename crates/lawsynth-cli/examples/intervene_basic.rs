use lawsynth_cli::parse_scheduled_assignment_text;
fn main() {
    let (time, id, value) = parse_scheduled_assignment_text("1.25:gain=2.5").unwrap();
    println!("at t={time}, set {id}={value}");
}
