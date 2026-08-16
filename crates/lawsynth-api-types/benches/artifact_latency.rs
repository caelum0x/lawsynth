use lawsynth_api_types::{ArtifactDescriptor, ArtifactId, ArtifactMediaType, ProjectId};
use std::hint::black_box;

fn main() {
    let project = ProjectId::parse("bench").unwrap();
    for index in 0..100_000 {
        black_box(
            ArtifactDescriptor::new(
                ArtifactId::parse(format!("artifact-{index}")).unwrap(),
                project.clone(),
                None,
                ArtifactMediaType::Json,
                index,
                "c".repeat(64),
            )
            .unwrap(),
        );
    }
}
