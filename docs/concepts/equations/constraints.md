# Constraints

`ExpressionConfig` limits expression depth and node count before an expression enters a bounded workflow. `WorldConfig` can validate symbols and unit dimensions. Discovery also validates dataset, feature, candidate, and resource limits before fitting.

These checks bound structure and reject malformed inputs. They do not prove physical admissibility, positivity, stability, conservation, or identifiability.

Encode domain constraints through preprocessing, feasible initial conditions, and independent scientific validation until the IR gains constraint semantics.
