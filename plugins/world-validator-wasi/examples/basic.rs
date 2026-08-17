//! Validate a world described in the plugin's line-oriented grammar.
//!
//! ```bash
//! cargo run --example basic
//! ```

use lawsynth_world_validator::WorldValidator;

fn main() {
    let world = "\
# A damped oscillator.
var x = 1.0
var v = 0.0
d(x)/dt = v
d(v)/dt = -x - 0.1 * v
";

    match WorldValidator::new().validate_text(world) {
        Ok(report) => {
            println!("valid world with {} variables", report.variable_count);
            for warning in &report.warnings {
                println!("warning: {warning}");
            }
        }
        Err(error) => println!("invalid world: {error}"),
    }
}
