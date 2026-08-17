//! The worker's produced-artifact handoff.
//!
//! When a job completes, the worker records a durable, content-addressed
//! *manifest* of its output bundle in the object store (production architecture,
//! sections 10 and 23: "worker uploads content-addressed artifacts"). The
//! executable payload itself stays typed and in-memory -- this crate does not
//! invent a wire codec for it -- so what is recorded is a deterministic summary
//! of the output: its kind, the number of items produced, and a human-readable
//! detail line. The manifest is written through [`crate::upload`], so it is only
//! finalized after checksum verification.

use lawsynth_store::{ObjectKey, ObjectStore};

use crate::{JobOutput, WorkerError, execute, upload::UploadReceipt};

/// The object-store prefix under which artifact manifests are recorded.
const PREFIX: &str = "worker/artifacts/";

/// A deterministic description of a completed job's output bundle.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactManifest {
    pub job_id: String,
    pub kind: &'static str,
    pub items: u64,
    pub summary: String,
}

impl ArtifactManifest {
    /// Summarizes a job output into a recordable manifest.
    pub fn from_output(job_id: impl Into<String>, output: &JobOutput) -> Self {
        let (kind, items) = match output {
            JobOutput::Discovery(result) => ("discover", result.candidates.len() as u64),
            JobOutput::Simulation(trajectory) => ("simulate", trajectory.samples() as u64),
        };
        Self { job_id: job_id.into(), kind, items, summary: execute::output_summary(output) }
    }

    fn key(&self) -> Result<ObjectKey, WorkerError> {
        ObjectKey::new(format!("{PREFIX}{}/manifest", self.job_id)).map_err(WorkerError::from)
    }

    fn encode(&self) -> Vec<u8> {
        format!(
            "version=1\njob_id={}\nkind={}\nitems={}\nsummary={}\n",
            self.job_id,
            self.kind,
            self.items,
            hex(&self.summary),
        )
        .into_bytes()
    }
}

/// Proof of a recorded artifact manifest: the job, its kind, and the verified
/// upload receipt for the stored bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactReceipt {
    pub job_id: String,
    pub kind: &'static str,
    pub items: u64,
    pub upload: UploadReceipt,
}

/// Records a completed job's output manifest to the object store, verified by
/// checksum, and returns the receipt.
pub(crate) fn record<S: ObjectStore>(
    store: &S,
    job_id: &str,
    output: &JobOutput,
    maximum_bytes: usize,
) -> Result<ArtifactReceipt, WorkerError> {
    let manifest = ArtifactManifest::from_output(job_id, output);
    let key = manifest.key()?;
    let receipt = crate::upload::upload(store, key, manifest.encode(), maximum_bytes)?;
    Ok(ArtifactReceipt {
        job_id: manifest.job_id,
        kind: manifest.kind,
        items: manifest.items,
        upload: receipt,
    })
}

fn hex(value: &str) -> String {
    value.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}
