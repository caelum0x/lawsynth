use crate::{ArtifactError, ArtifactService};

/// Observable local readiness state. A healthy report proves catalog records can be listed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthReport {
    pub artifact_count: usize,
    pub stored_data_bytes: u64,
    pub capacity_bytes: u64,
}

pub(crate) fn check(service: &ArtifactService) -> Result<HealthReport, ArtifactError> {
    Ok(HealthReport {
        artifact_count: service.catalog().count()?,
        stored_data_bytes: service.storage().stored_data_bytes()?,
        capacity_bytes: service.config().limits.max_total_bytes,
    })
}
