# Structural uncertainty

`StructuralUncertainty::new` accepts one or more sources only when every source is tagged `SourceKind::Structural` and has a finite non-negative standard deviation. Empty input returns `EmptyInput`; any wrong kind or invalid deviation returns `NonFiniteValue`.

`combined_standard_deviation` computes the root-sum-square of source deviations. That is an independence aggregation convention, not evidence that model alternatives are statistically independent.

`structural_score` normalizes Akaike-style weights from at least two finite candidate scores and returns `1 - Σ wᵢ²`. It measures concentration among supplied scores; it does not select a causal structure or calculate a posterior probability.
