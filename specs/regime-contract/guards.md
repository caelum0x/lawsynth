# Segmentation guards

`SegmentationConfig` requires finite `penalty >= 0` and `min_segment_len > 0`. `pelt` additionally requires non-empty data with at least `min_segment_len` observations. Violations return `InvalidParameter`, `EmptySeries`, or `InsufficientSamples` as applicable.

`segment_moments(data, start, end)` accepts only a valid half-open range within the slice. It checks each visited observation for finiteness and reports the original index for a bad value. The SSE is clamped at zero only to remove negligible floating-point cancellation.

No automatic scaling, detrending, penalty selection, or missing-data handling occurs before segmentation.
