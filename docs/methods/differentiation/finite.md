# Finite differences

`differentiate_series` uses a forward difference at the first sample, a backward difference at the last, and the three-point Lagrange derivative in the interior. The interior formula uses the adjacent left and right spacings, so it is exact for a quadratic evaluated on a non-uniform grid when the grid is valid.

Call `irregular_three_point_derivative` when input validation matters: it requires finite values and strictly increasing finite times. The lower-level function assumes usable spacings and therefore should not be used as a sanitizer. This is local numerical differentiation, not smoothing; measurement noise is amplified.
