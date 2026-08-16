# lawsynth-units

Deterministic SI dimensional-analysis utilities for variables, parameters, and
expressions. Built-in SI units cover common base and derived units, while a
registry supports application-local names.

## Use

```rust
use lawsynth_units::{convert, parse_unit};

let metres = parse_unit("m")?;
let centimetres = parse_unit("cm")?;
assert_eq!(convert(2.0, &metres, &centimetres)?, 200.0);
# Ok::<(), lawsynth_units::UnitError>(())
```

`require_compatible` rejects incompatible conversions. Unit inference tracks
dimensions through expression operators, but it does not infer physical
meaning, offsets, or uncertainty; callers must attach those domain assumptions.
