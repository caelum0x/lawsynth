//! Worker-pool definition and the registry that tracks their live reservations.
//!
//! A [`WorkerPool`] is validated metadata: a URL-safe id and a resource capacity.
//! The [`PoolRegistry`] owns the mutable side — the running reserved total per
//! pool — and delegates every arithmetic decision to [`crate::quota`], so the
//! scheduler never open-codes reservation math. The registry is the single owner
//! of pool state extracted out of the scheduler core.

use std::collections::BTreeMap;

use lawsynth_runner::ResourceRequest;

use crate::{SchedulerError, quota};

/// A named, resource-bounded worker pool available to the local scheduler.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerPool {
    pub id: String,
    pub capacity: ResourceRequest,
}

impl WorkerPool {
    pub fn new(id: impl Into<String>, capacity: ResourceRequest) -> Result<Self, SchedulerError> {
        let id = id.into();
        if id.is_empty()
            || id.len() > 128
            || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return Err(SchedulerError::InvalidWorker(
                "id must be URL-safe and no longer than 128 bytes".into(),
            ));
        }
        Ok(Self { id, capacity })
    }
}

/// A pool plus the resources currently reserved by leased jobs assigned to it.
#[derive(Clone, Debug)]
struct PoolState {
    pool: WorkerPool,
    reserved: ResourceRequest,
}

/// Registry of every worker pool the scheduler may place work on.
///
/// Registration is unique, and the running reservation per pool is adjusted only
/// through [`PoolRegistry::reserve`] / [`PoolRegistry::release`], which route the
/// checked arithmetic through [`crate::quota`].
#[derive(Clone, Debug, Default)]
pub struct PoolRegistry {
    pools: BTreeMap<String, PoolState>,
}

impl PoolRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a pool, rejecting a duplicate id as an invalid worker.
    pub fn register(&mut self, pool: WorkerPool) -> Result<(), SchedulerError> {
        if self.pools.contains_key(&pool.id) {
            return Err(SchedulerError::InvalidWorker(format!(
                "pool '{}' is already registered",
                pool.id
            )));
        }
        self.pools.insert(pool.id.clone(), PoolState { pool, reserved: quota::zero() });
        Ok(())
    }

    pub fn contains(&self, id: &str) -> bool {
        self.pools.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.pools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pools.is_empty()
    }

    /// Ids of every registered pool, in stable ascending order.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.pools.keys().map(String::as_str)
    }

    pub fn capacity(&self, id: &str) -> Option<ResourceRequest> {
        self.pools.get(id).map(|state| state.pool.capacity)
    }

    pub fn reserved(&self, id: &str) -> Option<ResourceRequest> {
        self.pools.get(id).map(|state| state.reserved)
    }

    /// Resources still free in a pool: `capacity - reserved`.
    pub fn available(&self, id: &str) -> Result<ResourceRequest, SchedulerError> {
        let state = self.state(id)?;
        quota::available(state.pool.capacity, state.reserved, id)
    }

    /// Admits `request` into a pool, growing its reservation.
    pub fn reserve(&mut self, id: &str, request: ResourceRequest) -> Result<(), SchedulerError> {
        let capacity = self.state(id)?.pool.capacity;
        let reserved = self.state(id)?.reserved;
        let next = quota::reserve(reserved, capacity, request, id)?;
        self.pools.get_mut(id).expect("pool exists").reserved = next;
        Ok(())
    }

    /// Releases `request` from a pool, shrinking its reservation.
    pub fn release(&mut self, id: &str, request: ResourceRequest) -> Result<(), SchedulerError> {
        let reserved = self.state(id)?.reserved;
        let next = quota::release(reserved, request, id)?;
        self.pools.get_mut(id).expect("pool exists").reserved = next;
        Ok(())
    }

    fn state(&self, id: &str) -> Result<&PoolState, SchedulerError> {
        self.pools.get(id).ok_or_else(|| SchedulerError::UnknownWorker(id.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(id: &str) -> WorkerPool {
        WorkerPool::new(id, ResourceRequest::new(500, 4096, 4096).unwrap()).unwrap()
    }

    fn request() -> ResourceRequest {
        ResourceRequest::new(250, 1024, 1024).unwrap()
    }

    #[test]
    fn rejects_ids_that_are_not_url_safe() {
        assert!(WorkerPool::new("has space", request()).is_err());
        assert!(WorkerPool::new("", request()).is_err());
    }

    #[test]
    fn registration_is_unique() {
        let mut registry = PoolRegistry::new();
        registry.register(pool("cpu-a")).unwrap();
        let error = registry.register(pool("cpu-a")).unwrap_err();
        assert!(matches!(error, SchedulerError::InvalidWorker(_)));
        assert!(registry.contains("cpu-a"));
        assert_eq!(registry.len(), 1);
    }

    #[test]
    fn reserve_then_release_round_trips_availability() {
        let mut registry = PoolRegistry::new();
        registry.register(pool("cpu-a")).unwrap();
        registry.reserve("cpu-a", request()).unwrap();
        assert_eq!(
            registry.available("cpu-a").unwrap(),
            ResourceRequest::new(250, 3072, 3072).unwrap()
        );
        registry.release("cpu-a", request()).unwrap();
        assert_eq!(
            registry.available("cpu-a").unwrap(),
            ResourceRequest::new(500, 4096, 4096).unwrap()
        );
    }

    #[test]
    fn unknown_pool_is_reported() {
        let registry = PoolRegistry::new();
        assert!(matches!(registry.available("ghost"), Err(SchedulerError::UnknownWorker(_))));
    }
}
