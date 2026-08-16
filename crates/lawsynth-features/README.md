# lawsynth-features

Typed, deterministic candidate feature libraries for sparse and symbolic
discovery. Libraries can include polynomial, trigonometric, bounded rational,
lagged, and interaction terms, subject to explicit structural constraints.

## Use

```rust
use lawsynth_core::Identifier;
use lawsynth_features::FeatureLibrary;

let library = FeatureLibrary::polynomial(
    [Identifier::new("x")?, Identifier::new("u")?], 2, true,
)?;
assert!(!library.terms().is_empty());
# Ok::<(), lawsynth_features::FeatureError>(())
```

Feature expansion is intentionally finite. Use `FeatureConstraint` to exclude
forbidden symbols or self-interactions and inspect the generated terms before
fitting. This crate does not estimate coefficients; pass its matrices to
`lawsynth-sparse` or another calibrated solver.
