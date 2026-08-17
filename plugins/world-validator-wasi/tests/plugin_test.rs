use lawsynth_plugin_api::PluginError;
use lawsynth_world_validator::{WorldSpec, WorldValidator};

#[test]
fn validates_a_well_formed_world() {
    let text = "\
var x = 1.0
var v = 0.0
d(x)/dt = v
d(v)/dt = -x - 0.1 * v
";
    let report = WorldValidator::new().validate_text(text).unwrap();
    assert_eq!(report.variable_count, 2);
    assert!(report.warnings.is_empty());
}

#[test]
fn rejects_missing_derivative() {
    let text = "var x = 1.0\n";
    let error = WorldValidator::new().validate_text(text).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn rejects_derivative_for_undeclared_variable() {
    let text = "var x = 1.0\nd(x)/dt = x\nd(y)/dt = x\n";
    let error = WorldValidator::new().validate_text(text).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn rejects_non_finite_initial_value_in_spec() {
    let spec = WorldSpec {
        variables: vec!["x".into()],
        initial_state: vec![f64::NAN],
        derivatives: vec!["x".into()],
    };
    let error = WorldValidator::new().validate(&spec).unwrap_err();
    assert!(matches!(error, PluginError::InvalidData(_)));
}

#[test]
fn wasi_entrypoint_returns_zero_for_valid_world() {
    let text = b"var x = 1.0\nd(x)/dt = -x\n";
    // SAFETY: the pointer and length describe a live, readable byte slice.
    let code = unsafe {
        lawsynth_world_validator::lawsynth_world_validate(text.as_ptr(), text.len())
    };
    assert_eq!(code, 0);
}

#[test]
fn wasi_entrypoint_rejects_null_pointer() {
    // SAFETY: a null pointer is explicitly handled by the entrypoint.
    let code = unsafe { lawsynth_world_validator::lawsynth_world_validate(std::ptr::null(), 0) };
    assert_eq!(code, -1);
}
