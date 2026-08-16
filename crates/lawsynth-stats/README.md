# lawsynth-stats

Small deterministic statistical primitives used throughout profiling,
discovery, and uncertainty summaries. It favors explicit validation and stable
ordering over hidden randomness or broad distribution fitting.

## Use

```rust
use lawsynth_stats::{moments, percentile_interval};

let summary = moments(&[1.0, 2.0, 3.0])?;
assert_eq!(summary.mean, 2.0);
let interval = percentile_interval(&[0.1, 0.2, 0.3, 0.4], 0.1)?;
assert!(interval.lower <= interval.upper);
# Ok::<(), lawsynth_stats::StatsError>(())
```

The crate includes quantiles, covariance, robust statistics, bootstrap index
generation, sampling, basic normal density/CDF, and histogram mutual
information. Inputs must be finite and sufficiently sized for each operation.
