use lawsynth_egraph::{RewriteConfig, RewriteSchedule};

#[test]
fn schedule_preserves_configured_pass_bound() {
    assert_eq!(RewriteSchedule::from_config(&RewriteConfig { max_passes: 3 }).unwrap().passes, 3);
}
