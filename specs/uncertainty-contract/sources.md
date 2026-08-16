# Declared uncertainty sources

`UncertaintySource` records a caller-supplied name, `SourceKind`, and standard deviation. `SourceKind` is one of measurement, parameter, structural, numerical, or sampling; the enum is provenance, not an estimator.

`UncertaintySource::variance` is the literal square of `standard_deviation`. It performs no validation because a standalone source is a plain data structure. Validation is enforced by aggregate constructors that accept sources.

There is no global source registry, automatic provenance inference, covariance between named sources, or unit conversion.
