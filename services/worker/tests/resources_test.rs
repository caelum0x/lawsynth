use lawsynth_runner::ResourceRequest;
use lawsynth_worker::{Job, JobEnvelope};

#[test]
fn typed_envelopes_inherit_runner_resource_and_deadline_validation() {
    let resources = ResourceRequest::new(1, 1024, 0).unwrap();
    let result = JobEnvelope::new(
        "🔒",
        0,
        10,
        10,
        resources,
        Job::Discover {
            dataset: lawsynth_data::Dataset::new(
                lawsynth_data::TimeAxis::new(vec![0.0, 1.0, 2.0]).unwrap(),
                [lawsynth_data::NumericColumn::new(
                    lawsynth_core::Identifier::new("x").unwrap(),
                    vec![0.0, 1.0, 2.0],
                )],
            )
            .unwrap(),
            config: lawsynth_discovery::DiscoveryConfig::new([
                lawsynth_core::Identifier::new("x").unwrap()
            ]),
        },
    );
    assert!(result.is_err());
}
