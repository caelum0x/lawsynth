# Change points

`pelt(data, SegmentationConfig)` minimizes a penalized sum of within-segment squared errors subject to a minimum segment length. It returns contiguous `Segment` values, an objective, and derived change-point indices. The implementation keeps the exact dynamic-programming recurrence rather than applying unproven pruning.

`best_binary_split` evaluates one split candidate. Use it when you need a local diagnostic rather than a full penalized segmentation.

The implemented cost targets scalar mean shifts. It does not estimate variance, trend, multivariate, seasonal, or nonlinear changes.
