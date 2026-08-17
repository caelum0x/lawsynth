//! Fair job selection across worker pools.
//!
//! [`crate::Scheduler::lease_next`] leases for one pool at a time, but a
//! dispatcher driving several pools needs to spread grants so no pool starves.
//! [`FairShare`] provides that policy: it tracks how many jobs each pool has been
//! granted and always nominates the least-served eligible pool next, breaking
//! ties by pool id for determinism. This is a genuine weighted round-robin the
//! scheduler exposes for multi-pool dispatch; it holds only counters, so it is
//! trivially testable and reproducible.

use std::collections::BTreeMap;

/// Tracks per-pool grant counts to nominate the least-served pool next.
#[derive(Clone, Debug, Default)]
pub struct FairShare {
    granted: BTreeMap<String, u64>,
}

impl FairShare {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers pools so they are eligible even before their first grant.
    pub fn with_pools<I, S>(pools: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut share = Self::new();
        for pool in pools {
            share.granted.entry(pool.into()).or_insert(0);
        }
        share
    }

    /// Ensures `pool` is eligible for selection.
    pub fn register(&mut self, pool: impl Into<String>) {
        self.granted.entry(pool.into()).or_insert(0);
    }

    /// The least-served eligible pool, ties broken by ascending id.
    ///
    /// Returns `None` only when no pools are registered. Because [`BTreeMap`]
    /// iterates in ascending key order, the first pool reaching the running
    /// minimum wins, making the tie-break deterministic.
    pub fn next_pool(&self) -> Option<&str> {
        self.granted
            .iter()
            .min_by_key(|(id, count)| (**count, (*id).clone()))
            .map(|(id, _)| id.as_str())
    }

    /// Records that `pool` received a grant, increasing its served count.
    pub fn record_grant(&mut self, pool: impl Into<String>) {
        let entry = self.granted.entry(pool.into()).or_insert(0);
        *entry = entry.saturating_add(1);
    }

    /// The number of grants recorded for `pool`.
    pub fn grants(&self, pool: &str) -> u64 {
        self.granted.get(pool).copied().unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_share_nominates_nothing() {
        assert_eq!(FairShare::new().next_pool(), None);
    }

    #[test]
    fn spreads_grants_evenly_across_pools() {
        let mut share = FairShare::with_pools(["a", "b", "c"]);
        let mut order = Vec::new();
        for _ in 0..6 {
            let pool = share.next_pool().unwrap().to_owned();
            share.record_grant(&pool);
            order.push(pool);
        }
        // Two full rounds, each visiting every pool once, id-ordered on ties.
        assert_eq!(order, vec!["a", "b", "c", "a", "b", "c"]);
        assert_eq!(share.grants("a"), 2);
        assert_eq!(share.grants("b"), 2);
        assert_eq!(share.grants("c"), 2);
    }

    #[test]
    fn a_lagging_pool_is_served_first() {
        let mut share = FairShare::with_pools(["a", "b"]);
        share.record_grant("a");
        share.record_grant("a");
        assert_eq!(share.next_pool(), Some("b"));
    }
}
