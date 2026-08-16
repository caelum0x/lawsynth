# lawsynth-preprocess

Reproducible numerical transforms for aligned time-series data. The crate
provides imputation, linear resampling/alignment, detrending, standardization,
moving-average smoothing, and an auditable pipeline of applied steps.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_preprocess::{moving_average, standardize};

let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0, 2.0, 3.0])?,
    [NumericColumn::new(Identifier::new("x")?, vec![1.0, 3.0, 5.0, 7.0])])?;
let (smoothed, _) = moving_average(&data, 1)?;
let (scaled, report) = standardize(&smoothed)?;
assert_eq!(scaled.len(), 4);
assert!(report.standard_deviation["x"] > 0.0);
# Ok::<(), lawsynth_preprocess::PreprocessError>(())
```

Every transform reports its fitted values so it can be replayed on evaluation
data. Boundary interpolation and imputation are modeling choices; this crate
records them but cannot make them scientifically neutral.
