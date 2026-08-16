# Uncertainty sources

`UncertaintySource` records a source kind, standard deviation, and weight. Source kinds distinguish measurement, parameter, structural, and numerical contributions. `StructuralUncertainty` validates finite non-negative magnitudes and combines weighted independent contributions in quadrature.

This representation records an uncertainty budget. It does not estimate a source from raw data or prove that sources are independent.
