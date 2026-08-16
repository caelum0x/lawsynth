# lawsynth-profile

Deterministic quality and distribution summaries for numeric datasets. Profiles
capture missingness, moments, sampling properties, robust quality flags,
pairwise correlation, and bounded delay estimates before discovery changes data.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_data::{Dataset, NumericColumn, TimeAxis};
use lawsynth_profile::profile;

let data = Dataset::new(TimeAxis::new(vec![0.0, 1.0, 2.0])?,
    vec![NumericColumn::new(Identifier::new("signal")?, vec![1.0, 2.0, 3.0])])?;
let report = profile(&data)?;
assert_eq!(report.columns.len(), 1);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Profiles are descriptive evidence, not causal conclusions. Preserve them with
the discovery configuration so later users can see data-quality assumptions.
