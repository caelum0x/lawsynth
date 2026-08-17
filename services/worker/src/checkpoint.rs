use lawsynth_store::{ObjectKey, ObjectStore};

use crate::WorkerError;

const PREFIX: &str = "worker/checkpoints/";

/// Durable lifecycle state. A checkpoint records an observed state transition,
/// not a claim that arbitrary job payloads can be serialized and resumed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CheckpointState {
    Running,
    Completed,
    Failed,
    Cancelled,
    Rejected,
}

impl CheckpointState {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
            Self::Rejected => "rejected",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "running" => Self::Running,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "cancelled" => Self::Cancelled,
            "rejected" => Self::Rejected,
            _ => return None,
        })
    }
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Running)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JobCheckpoint {
    pub job_id: String,
    pub sequence: u64,
    pub recorded_at_ms: u64,
    pub state: CheckpointState,
    pub detail: String,
}

impl JobCheckpoint {
    pub(crate) fn key(job_id: &str) -> ObjectKey {
        ObjectKey::new(format!("{PREFIX}{job_id}.checkpoint"))
            .expect("validated work ID produces an object-store-safe checkpoint key")
    }

    pub(crate) fn encode(&self) -> Vec<u8> {
        format!(
            "version=1\njob_id={}\nsequence={}\nrecorded_at_ms={}\nstate={}\ndetail={}\n",
            self.job_id,
            self.sequence,
            self.recorded_at_ms,
            self.state.as_str(),
            hex(&self.detail),
        )
        .into_bytes()
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, WorkerError> {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| WorkerError::CorruptCheckpoint("checkpoint is not UTF-8".into()))?;
        let mut fields = std::collections::BTreeMap::new();
        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                return Err(WorkerError::CorruptCheckpoint("malformed checkpoint line".into()));
            };
            if fields.insert(key, value).is_some() {
                return Err(WorkerError::CorruptCheckpoint("duplicate checkpoint field".into()));
            }
        }
        if fields.get("version") != Some(&"1") {
            return Err(WorkerError::CorruptCheckpoint("unsupported checkpoint version".into()));
        }
        let job_id = required(&fields, "job_id")?.to_owned();
        let sequence = parse_number(&fields, "sequence")?;
        let recorded_at_ms = parse_number(&fields, "recorded_at_ms")?;
        let state = CheckpointState::parse(required(&fields, "state")?)
            .ok_or_else(|| WorkerError::CorruptCheckpoint("unknown lifecycle state".into()))?;
        let detail = unhex(required(&fields, "detail")?)?;
        if fields.len() != 6 || job_id.is_empty() {
            return Err(WorkerError::CorruptCheckpoint("unexpected checkpoint fields".into()));
        }
        Ok(Self { job_id, sequence, recorded_at_ms, state, detail })
    }
}

pub(crate) fn load<S: ObjectStore>(
    store: &S,
    job_id: &str,
) -> Result<Option<JobCheckpoint>, WorkerError> {
    match store.get(&JobCheckpoint::key(job_id)) {
        Ok(object) => JobCheckpoint::decode(&object.bytes).map(Some),
        Err(lawsynth_store::StoreError::NotFound(_)) => Ok(None),
        Err(error) => Err(error.into()),
    }
}

/// Enumerates the ids of every job with a durable checkpoint by listing the
/// object store under the checkpoint prefix. This is the read path the status
/// transport uses to advertise known jobs; the store, not an in-memory index,
/// remains the authority.
pub(crate) fn list<S: ObjectStore>(store: &S) -> Result<Vec<String>, WorkerError> {
    let keys = store.list(Some(PREFIX))?;
    let mut ids = Vec::new();
    for key in keys {
        if let Some(rest) = key.as_str().strip_prefix(PREFIX) {
            if let Some(job_id) = rest.strip_suffix(".checkpoint") {
                if !job_id.is_empty() {
                    ids.push(job_id.to_owned());
                }
            }
        }
    }
    Ok(ids)
}

pub(crate) fn save<S: ObjectStore>(
    store: &S,
    record: &JobCheckpoint,
    maximum_bytes: usize,
) -> Result<(), WorkerError> {
    let bytes = record.encode();
    if bytes.len() > maximum_bytes {
        return Err(WorkerError::InvalidJob("checkpoint exceeds configured size limit".into()));
    }
    store.put(JobCheckpoint::key(&record.job_id), bytes)?;
    Ok(())
}

fn required<'a>(
    fields: &'a std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Result<&'a str, WorkerError> {
    fields
        .get(key)
        .copied()
        .ok_or_else(|| WorkerError::CorruptCheckpoint(format!("missing '{key}'")))
}
fn parse_number(
    fields: &std::collections::BTreeMap<&str, &str>,
    key: &str,
) -> Result<u64, WorkerError> {
    required(fields, key)?
        .parse()
        .map_err(|_| WorkerError::CorruptCheckpoint(format!("invalid '{key}'")))
}
fn hex(value: &str) -> String {
    value.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect()
}
fn unhex(value: &str) -> Result<String, WorkerError> {
    if value.len() % 2 != 0 {
        return Err(WorkerError::CorruptCheckpoint("odd hexadecimal detail".into()));
    }
    let bytes = (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| WorkerError::CorruptCheckpoint("invalid hexadecimal detail".into()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes)
        .map_err(|_| WorkerError::CorruptCheckpoint("detail is not UTF-8".into()))
}
