# Algorithm contributions

Numerical code must state its input domain, deterministic behavior,
tolerances, and failure conditions. The current discovery path accepts finite
numeric time series, estimates derivatives, builds scalar features, performs
sparse regression, scores candidates, and renders symbolic expressions.

Add a focused unit test for invariants and an integration/scientific test for
observable behavior. Fixtures must be generated from defined equations or
recorded measurements with provenance; do not encode expected output from the
implementation under test. Compare floating-point results using a stated
tolerance and include conditioning or sampling assumptions.

Adding a method means wiring it through its configuration, validation,
selection logic, diagnostics, documentation, and CLI/Python surface where
appropriate. A method that cannot meet these requirements is not a supported
option.
