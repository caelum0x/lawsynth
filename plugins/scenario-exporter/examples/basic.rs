//! Export a discovered scenario to JSON and to the world grammar.
//!
//! ```bash
//! cargo run --example basic
//! ```

use lawsynth_scenario_exporter::{ExportFormat, Scenario, ScenarioExporter};

fn main() {
    let scenario = Scenario {
        id: "damped-oscillator".into(),
        variables: vec!["x".into(), "v".into()],
        initial_state: vec![1.0, 0.0],
        laws: vec!["v".into(), "-x - 0.1 * v".into()],
    };

    let exporter = ScenarioExporter::new();

    let json = exporter
        .export(&scenario, ExportFormat::Json)
        .expect("valid scenario exports to JSON");
    println!("--- {} ---\n{}\n", json.media_type, json.content);

    let world = exporter
        .export(&scenario, ExportFormat::World)
        .expect("valid scenario exports to the world grammar");
    println!("--- {} ---\n{}", world.media_type, world.content);
}
