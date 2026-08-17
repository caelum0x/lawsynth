//! Plugin-seam tests. This build links no plugin runtime, so the seam validates
//! a dispatch request and then reports an honest, un-faked "not linked" outcome.

use lawsynth_worker::{PluginDispatch, PluginKind, PluginRequest, PluginSeam, WorkerError};

#[test]
fn a_valid_request_is_constructed_and_describes_its_target() {
    let request =
        PluginRequest::new("sindy.v2", PluginKind::Algorithm, vec!["read-data".to_string()])
            .unwrap();
    assert_eq!(request.plugin_id, "sindy.v2");
    assert_eq!(request.kind.as_str(), "algorithm");
    assert_eq!(request.capabilities, vec!["read-data".to_string()]);
}

#[test]
fn invalid_plugin_ids_and_capabilities_are_rejected() {
    assert!(matches!(
        PluginRequest::new("bad id", PluginKind::Simulator, Vec::new()),
        Err(WorkerError::Plugin(_))
    ));
    assert!(matches!(
        PluginRequest::new("ok", PluginKind::Simulator, vec![String::new()]),
        Err(WorkerError::Plugin(_))
    ));
    assert!(matches!(
        PluginRequest::new("ok", PluginKind::Simulator, vec!["null\0byte".to_string()]),
        Err(WorkerError::Plugin(_))
    ));
}

#[test]
fn the_default_seam_reports_not_linked_and_never_fakes_execution() {
    let seam = PluginSeam::new();
    assert!(!seam.is_linked());

    let request = PluginRequest::new("adapter.csv", PluginKind::DataAdapter, Vec::new()).unwrap();
    let outcome = seam.dispatch(&request);
    // Honest failure naming the missing runtime; no success is ever fabricated.
    let error = outcome.expect_err("an unlinked seam cannot succeed");
    let WorkerError::Plugin(message) = error else {
        panic!("expected a plugin error, got {error:?}");
    };
    assert!(message.contains("adapter.csv"));
    assert!(message.contains("lawsynth-plugin-host"));
    assert!(PluginSeam::describe().contains("not linked"));
}
