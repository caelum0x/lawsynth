# Change-point algorithms

`pelt(data, config)` implements the exact dynamic-programming recurrence for penalized within-segment sum of squared error. Despite its name, this version intentionally does not apply pruning that could change results without a proved pruning condition. Every output segment has length at least `min_segment_len`.

`best_binary_split` evaluates every legal single cut and returns the greatest gain `full_cost - left_cost - right_cost`; ties retain the earliest encountered cut. If the series is shorter than two legal segments it returns `Ok(None)`.

Both operations reject non-finite data. Neither infers a causal mechanism or supports multivariate, weighted, robust, or missing-data costs.
