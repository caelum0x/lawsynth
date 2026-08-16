use lawsynth_api_types::{ProjectId, RunId, RunStatus, RunSummary};

#[test]
fn terminal_status_requires_completion_timestamp() {
    let project = ProjectId::parse("project").unwrap();
    let run = RunId::parse("run-1").unwrap();
    assert!(RunSummary::new(run.clone(), project.clone(), RunStatus::Queued, 1, None).is_ok());
    assert!(RunSummary::new(run.clone(), project.clone(), RunStatus::Succeeded, 1, None).is_err());
    assert!(RunSummary::new(run, project, RunStatus::Succeeded, 2, Some(1)).is_err());
    assert!(RunStatus::Queued.can_transition_to(RunStatus::Running));
    assert!(!RunStatus::Succeeded.can_transition_to(RunStatus::Running));
}
