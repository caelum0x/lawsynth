use crate::{Artifact, ArtifactId};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
struct Entry {
    artifact: Artifact,
    touched: u64,
}

/// Process-local LRU for verified reads. It never acts as an authority for metadata.
#[derive(Clone, Debug)]
pub struct ArtifactCache {
    capacity_bytes: usize,
    used_bytes: usize,
    clock: u64,
    entries: BTreeMap<ArtifactId, Entry>,
}

impl ArtifactCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self { capacity_bytes, used_bytes: 0, clock: 0, entries: BTreeMap::new() }
    }

    pub fn get(&mut self, id: &ArtifactId) -> Option<Artifact> {
        self.clock = self.clock.wrapping_add(1);
        self.entries.get_mut(id).map(|entry| {
            entry.touched = self.clock;
            entry.artifact.clone()
        })
    }

    pub fn insert(&mut self, artifact: Artifact) {
        self.clock = self.clock.wrapping_add(1);
        if artifact.bytes.len() > self.capacity_bytes {
            self.remove(artifact.id());
            return;
        }
        let id = artifact.id().clone();
        if let Some(previous) = self.entries.insert(id, Entry { artifact, touched: self.clock }) {
            self.used_bytes -= previous.artifact.bytes.len();
        }
        self.recount();
        while self.used_bytes > self.capacity_bytes {
            let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.touched)
                .map(|(id, _)| id.clone())
            else {
                break;
            };
            self.entries.remove(&oldest);
            self.recount();
        }
    }

    pub fn remove(&mut self, id: &ArtifactId) {
        self.entries.remove(id);
        self.recount();
    }

    fn recount(&mut self) {
        self.used_bytes = self.entries.values().map(|entry| entry.artifact.bytes.len()).sum();
    }
}
