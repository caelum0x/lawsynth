# Parameter profiling

`profile_quadratic` fits the least-squares quadratic `a x² + b x + c` to at least three finite `ProfilePoint` observations. It returns the vertex, fitted minimum, curvature `a`, and a normal-approximation two-sided interval with radius `z / sqrt(a)`.

The normal equations are solved by pivoted elimination. Rank deficiency returns `SingularCovariance`; a non-positive curvature returns `NonPositiveVariance`. The interval configuration requires finite `0 < confidence < 1`.

This is a local quadratic approximation, not a general profile-likelihood optimizer. It neither constrains parameters nor evaluates a model objective itself.
