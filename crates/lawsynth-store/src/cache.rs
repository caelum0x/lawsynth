use crate::{Object, ObjectKey};
use std::collections::BTreeMap;
#[derive(Clone, Debug)]
struct Entry {
    object: Object,
    last_used: u64,
}
/// Bounded LRU cache of immutable objects. It never changes store ownership.
#[derive(Clone, Debug)]
pub struct ObjectCache {
    capacity_bytes: usize,
    used_bytes: usize,
    sequence: u64,
    entries: BTreeMap<ObjectKey, Entry>,
}
impl ObjectCache {
    pub fn new(capacity_bytes: usize) -> Self {
        Self {
            capacity_bytes,
            used_bytes: 0,
            sequence: 0,
            entries: BTreeMap::new(),
        }
    }
    pub fn get(&mut self, key: &ObjectKey) -> Option<Object> {
        self.sequence = self.sequence.wrapping_add(1);
        self.entries.get_mut(key).map(|entry| {
            entry.last_used = self.sequence;
            entry.object.clone()
        })
    }
    pub fn insert(&mut self, key: ObjectKey, object: Object) {
        self.sequence = self.sequence.wrapping_add(1);
        if object.len() > self.capacity_bytes {
            self.entries.remove(&key);
            self.recount();
            return;
        }
        if let Some(previous) = self.entries.insert(
            key,
            Entry {
                object,
                last_used: self.sequence,
            },
        ) {
            self.used_bytes -= previous.object.len();
        }
        self.recount();
        while self.used_bytes > self.capacity_bytes {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            {
                self.entries.remove(&oldest);
                self.recount();
            } else {
                break;
            }
        }
    }
    pub fn invalidate(&mut self, key: &ObjectKey) -> bool {
        let existed = self.entries.remove(key).is_some();
        self.recount();
        existed
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn used_bytes(&self) -> usize {
        self.used_bytes
    }
    fn recount(&mut self) {
        self.used_bytes = self.entries.values().map(|entry| entry.object.len()).sum();
    }
}
