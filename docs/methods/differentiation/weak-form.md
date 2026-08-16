# Weak-form integration

`weak_derivative_integral` is the implemented weak-form primitive: for aligned samples and a supplied test function it accumulates the trapezoidal approximation of `-∫ x(t) φ'(t) dt`, with endpoint terms handled according to the function API.

It is a numerical identity helper, not a full weak-form sparse-regression pipeline. Test-function design, boundary treatment, quadrature error control, and multi-equation assembly remain caller responsibilities.
