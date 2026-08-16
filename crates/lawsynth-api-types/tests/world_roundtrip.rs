use lawsynth_api_types::{
    ApiEvent, EventKind, ProjectId, RunId, WorldId, WorldRevision, validate_event_stream,
};

#[test]
fn revision_and_event_stream_preserve_ordering_invariants() {
    let project = ProjectId::parse("project").unwrap();
    let world = WorldRevision::new(
        project.clone(),
        WorldId::parse("world").unwrap(),
        1,
        "f".repeat(64),
    )
    .unwrap();
    assert_eq!(world.revision, 1);
    let run = RunId::parse("run").unwrap();
    let one = ApiEvent::new(
        1,
        10,
        project.clone(),
        Some(run.clone()),
        EventKind::RunQueued,
        "{}",
        64,
    )
    .unwrap();
    let two = ApiEvent::new(2, 10, project, Some(run), EventKind::RunStarted, "{}", 64).unwrap();
    assert!(validate_event_stream(&[one, two]).is_ok());
}
