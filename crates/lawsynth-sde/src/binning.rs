use lawsynth_core::Identifier;

use crate::{BinRule, BinnedEstimate, SdeError};

/// Deterministic per-bin accumulators for the conditional-moment estimates.
struct BinAccumulator {
    count: usize,
    sum_x: f64,
    sum_drift: f64,
    sum_diffusion: f64,
}

impl BinAccumulator {
    fn new() -> Self {
        Self { count: 0, sum_x: 0.0, sum_drift: 0.0, sum_diffusion: 0.0 }
    }
}

/// Estimates the raw binned Kramers–Moyal drift/diffusion table for one state.
///
/// `source` are the state values at the start of each increment (`X(t)`),
/// `increment` are the matching `ΔX = X(t+Δt) − X(t)`, and `dt` are the matching
/// per-step `Δt`. All three slices are aligned and equal length (`rows − 1`).
///
/// The state span `[min, max]` is partitioned according to `rule`; each source
/// value is assigned to a bin and its `ΔX/Δt` (drift) and `ΔX²/Δt` (diffusion)
/// contributions are averaged within the bin. Iteration order is fixed, so the
/// output is bit-identical for identical inputs.
pub(crate) fn binned_estimates(
    state: &Identifier,
    source: &[f64],
    increment: &[f64],
    dt: &[f64],
    rule: BinRule,
) -> Result<Vec<BinnedEstimate>, SdeError> {
    debug_assert_eq!(source.len(), increment.len());
    debug_assert_eq!(source.len(), dt.len());

    let (minimum, maximum) = span(source);
    // A strictly positive span is required to define finite-width bins. Using
    // `partial_cmp` keeps a NaN span (incomparable) on the error path too.
    if !matches!(maximum.partial_cmp(&minimum), Some(std::cmp::Ordering::Greater)) {
        return Err(SdeError::DegenerateState { state: state.clone() });
    }

    let (bin_count, width) = resolve_bins(rule, minimum, maximum);
    let mut bins = (0..bin_count).map(|_| BinAccumulator::new()).collect::<Vec<_>>();

    for ((&x, &dx), &step) in source.iter().zip(increment).zip(dt) {
        let index = bin_index(x, minimum, width, bin_count);
        let accumulator = &mut bins[index];
        accumulator.count += 1;
        accumulator.sum_x += x;
        accumulator.sum_drift += dx / step;
        accumulator.sum_diffusion += dx * dx / step;
    }

    Ok(bins
        .into_iter()
        .filter(|bin| bin.count > 0)
        .map(|bin| {
            let count = bin.count as f64;
            BinnedEstimate {
                x_center: bin.sum_x / count,
                drift: bin.sum_drift / count,
                diffusion: bin.sum_diffusion / count,
                count: bin.count,
            }
        })
        .collect())
}

/// The minimum and maximum of a non-empty slice, computed in fixed order.
fn span(values: &[f64]) -> (f64, f64) {
    let mut minimum = values[0];
    let mut maximum = values[0];
    for &value in &values[1..] {
        if value < minimum {
            minimum = value;
        }
        if value > maximum {
            maximum = value;
        }
    }
    (minimum, maximum)
}

/// Resolves a [`BinRule`] into a concrete bin count and (uniform) bin width.
fn resolve_bins(rule: BinRule, minimum: f64, maximum: f64) -> (usize, f64) {
    match rule {
        BinRule::Count(count) => (count.max(1), (maximum - minimum) / count.max(1) as f64),
        BinRule::Width(width) => {
            let span = maximum - minimum;
            let count = (span / width).ceil() as usize;
            (count.max(1), width)
        }
    }
}

/// Maps a state value to a bin index, clamped into `[0, bin_count)`.
fn bin_index(x: f64, minimum: f64, width: f64, bin_count: usize) -> usize {
    let raw = ((x - minimum) / width).floor();
    if raw < 0.0 { 0 } else { (raw as usize).min(bin_count - 1) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id(name: &str) -> Identifier {
        Identifier::new(name).unwrap()
    }

    #[test]
    fn averages_drift_and_diffusion_within_bins() {
        // Two clearly separated clusters -> two occupied bins.
        let source = vec![0.0, 0.0, 10.0, 10.0];
        let increment = vec![1.0, 3.0, -2.0, -4.0];
        let dt = vec![1.0, 1.0, 1.0, 1.0];
        let bins = binned_estimates(&id("x"), &source, &increment, &dt, BinRule::Count(2)).unwrap();
        assert_eq!(bins.len(), 2);
        // Low bin: drift mean = (1 + 3)/2 = 2, diffusion mean = (1 + 9)/2 = 5.
        assert_eq!(bins[0].x_center, 0.0);
        assert_eq!(bins[0].drift, 2.0);
        assert_eq!(bins[0].diffusion, 5.0);
        assert_eq!(bins[0].count, 2);
        // High bin: drift mean = (-2 + -4)/2 = -3, diffusion mean = (4 + 16)/2 = 10.
        assert_eq!(bins[1].x_center, 10.0);
        assert_eq!(bins[1].drift, -3.0);
        assert_eq!(bins[1].diffusion, 10.0);
    }

    #[test]
    fn rejects_a_degenerate_state() {
        let source = vec![5.0, 5.0, 5.0];
        let increment = vec![0.1, -0.1, 0.2];
        let dt = vec![1.0, 1.0, 1.0];
        let result = binned_estimates(&id("x"), &source, &increment, &dt, BinRule::Count(4));
        assert!(matches!(result, Err(SdeError::DegenerateState { .. })));
    }

    #[test]
    fn respects_per_step_dt() {
        // Same increment but different Δt must scale drift/diffusion.
        let source = vec![0.0, 1.0];
        let increment = vec![2.0, 2.0];
        let dt = vec![1.0, 4.0];
        let bins = binned_estimates(&id("x"), &source, &increment, &dt, BinRule::Count(2)).unwrap();
        assert_eq!(bins[0].drift, 2.0);
        assert_eq!(bins[0].diffusion, 4.0);
        assert_eq!(bins[1].drift, 0.5);
        assert_eq!(bins[1].diffusion, 1.0);
    }
}
