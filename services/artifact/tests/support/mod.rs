use lawsynth_artifact_service::{ArtifactConfig, ArtifactService};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

pub struct TestRoot {
    path: PathBuf,
}

impl TestRoot {
    pub fn new(label: &str) -> Self {
        let number = NEXT_ROOT.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir()
            .join(format!("lawsynth-artifact-{label}-{}-{number}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self { path }
    }
    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.path
    }
    #[allow(dead_code)]
    pub fn service(&self) -> ArtifactService {
        ArtifactService::open(ArtifactConfig::new(&self.path)).unwrap()
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
