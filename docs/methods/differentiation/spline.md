# Natural cubic spline

`cubic_spline_derivative` solves the tridiagonal natural-cubic-spline system, imposing zero second derivative at the two endpoints. It accepts finite, strictly increasing non-uniform times and returns derivatives at the knots.

Natural endpoint conditions influence derivatives near both boundaries; they are not inferred from the data. The method rejects fewer than three points, repeated/decreasing times, and non-finite observations. It does not expose alternative boundary conditions or spline uncertainty.
