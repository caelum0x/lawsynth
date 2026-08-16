use lawsynth_api_types::{Project, ProjectId};

#[test]
fn project_requires_portable_id_and_display_name() {
    let id = ProjectId::parse("wind-tunnel_01").unwrap();
    let project = Project::new(id.clone(), "Wind tunnel", 42).unwrap();
    assert_eq!(project.id, id);
    assert!(ProjectId::parse("contains spaces").is_err());
    assert!(Project::new(id, "   ", 42).is_err());
}
