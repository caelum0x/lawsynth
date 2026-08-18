use crate::{RegimeError, Result, Segment, Segmentation, SegmentationConfig, segment_moments};
/// Exact penalized least-squares segmentation.  The recurrence is the PELT
/// objective; it intentionally avoids unsafe pruning for arbitrary inputs.
pub fn pelt(data: &[f64], config: SegmentationConfig) -> Result<Segmentation> {
    let config = config.validate()?;
    if data.is_empty() {
        return Err(RegimeError::EmptySeries);
    }
    if data.len() < config.min_segment_len {
        return Err(RegimeError::InsufficientSamples {
            required: config.min_segment_len,
            actual: data.len(),
        });
    }
    for (i, &v) in data.iter().enumerate() {
        if !v.is_finite() {
            return Err(RegimeError::NonFiniteObservation { index: i });
        }
    }
    let n = data.len();
    let mut sums = vec![0.0; n + 1];
    let mut squares = vec![0.0; n + 1];
    for i in 0..n {
        sums[i + 1] = sums[i] + data[i];
        squares[i + 1] = squares[i] + data[i] * data[i];
    }
    let cost = |a: usize, b: usize| {
        let len = (b - a) as f64;
        (squares[b] - squares[a] - (sums[b] - sums[a]).powi(2) / len).max(0.0)
    };
    let mut best = vec![f64::INFINITY; n + 1];
    let mut prev = vec![0usize; n + 1];
    best[0] = -config.penalty;
    for end in config.min_segment_len..=n {
        for start in 0..=end - config.min_segment_len {
            if best[start].is_finite() {
                let objective = best[start] + config.penalty + cost(start, end);
                if objective < best[end] {
                    best[end] = objective;
                    prev[end] = start;
                }
            }
        }
    }
    if !best[n].is_finite() {
        return Err(RegimeError::InsufficientSamples {
            required: config.min_segment_len,
            actual: n,
        });
    }
    let mut ranges = Vec::new();
    let mut end = n;
    while end > 0 {
        let start = prev[end];
        ranges.push((start, end));
        end = start;
    }
    ranges.reverse();
    let segments = ranges
        .into_iter()
        .map(|(a, b)| {
            let m = segment_moments(data, a, b)?;
            Ok(Segment { start: a, end: b, mean: m.mean, sum_squared_error: m.sum_squared_error })
        })
        .collect::<Result<Vec<_>>>()?;
    Segmentation::new(segments, best[n], n)
}
