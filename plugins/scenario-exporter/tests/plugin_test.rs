use lawsynth_plugin_api::PluginError;
use lawsynth_scenario_exporter::{ExportFormat, Scenario, ScenarioExporter};

fn scenario() -> Scenario {
    Scenario {
        id: "damped-oscillator".into(),
        variables: vec!["x".into(), "v".into()],
        initial_state: vec![1.0, 0.0],
        laws: vec!["v".into(), "-x - 0.1 * v".into()],
    }
}

#[test]
fn json_export_is_deterministic() {
    let exporter = ScenarioExporter::new();
    let first = exporter.export(&scenario(), ExportFormat::Json).unwrap();
    let second = exporter.export(&scenario(), ExportFormat::Json).unwrap();
    assert_eq!(first.content, second.content);
    assert_eq!(first.media_type, "application/json");
    assert!(first.content.contains("\"id\": \"damped-oscillator\""));
}

#[test]
fn world_export_lists_vars_and_laws() {
    let artifact = ScenarioExporter::new()
        .export(&scenario(), ExportFormat::World)
        .unwrap();
    assert!(artifact.content.contains("var x = 1.0"));
    assert!(artifact.content.contains("d(v)/dt = -x - 0.1 * v"));
}

#[test]
fn rejects_empty_scenario_id() {
    let mut bad = scenario();
    bad.id = String::new();
    let error = ScenarioExporter::new()
        .export(&bad, ExportFormat::Json)
        .unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn rejects_mismatched_law_count() {
    let mut bad = scenario();
    bad.laws.pop();
    let error = ScenarioExporter::new()
        .export(&bad, ExportFormat::Json)
        .unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn json_escapes_special_characters() {
    let mut scenario = scenario();
    scenario.laws[0] = "a\"quote".into();
    let artifact = ScenarioExporter::new()
        .export(&scenario, ExportFormat::Json)
        .unwrap();
    assert!(artifact.content.contains("a\\\"quote"));
}
