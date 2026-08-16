# Time axis

`TimeAxis::new` requires a nonempty `Vec<f64>` of finite timestamps. For every
index after zero, the timestamp must be strictly greater than its predecessor;
duplicates and decreasing observations are rejected.

Sampling need not be regular. `TimeAxis::is_regular(relative_tolerance)` uses
the first interval as reference and accepts each subsequent interval when its
absolute difference is at most `max(abs(reference), 1) * tolerance`. For fewer
than three points it returns true; invalid (negative or non-finite) tolerance
returns true only for such short axes.

Discovery itself requires at least three samples. Its profiling output reports
start, end, nominal step `(end - start)/(n - 1)`, and regularity with the
profile configuration tolerance (default `1e-9`).
