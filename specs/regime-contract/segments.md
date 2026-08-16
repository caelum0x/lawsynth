# Segment invariants

A `Segment` denotes the half-open index range `[start, end)`, with `end > start`. `Segmentation::new` requires a non-empty contiguous sequence beginning at zero and ending exactly at the supplied observation count. Gaps, overlap, empty ranges, out-of-range ends, non-finite means, or negative SSE return an error.

`objective` must be finite. `change_points` returns each segment end except the final end; `label_at` returns the zero-based segment index for a covered observation and `None` outside the segmentation.

Segments summarize scalar data only. They do not carry timestamps, confidence values, multivariate covariance, or event causes.
