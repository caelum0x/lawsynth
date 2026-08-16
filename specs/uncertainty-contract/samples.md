# Validated samples

`Samples::new(Vec<f64>)` owns a non-empty one-dimensional sample. Construction rejects empty input with `EmptyInput` and every NaN or infinity with `NonFiniteValue`; consequently `as_slice`, `mean`, and downstream samplers never expose invalid observations.

`variance` is the unbiased sample variance, dividing by `n - 1`. It requires at least two values and otherwise returns `TooFewSamples { minimum: 2, actual: n }`. `standard_error` is `sqrt(variance / n)` and has the same cardinality requirement.

The type does not attach units, weights, timestamps, censoring metadata, or a probability-family interpretation. Callers needing any of those must model them outside this primitive.
