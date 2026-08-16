# Covariance

`CovarianceMatrix::from_observations` computes the unbiased sample covariance from finite row-major observations, requiring at least two equally wide rows. `from_row_major` accepts an explicitly symmetric finite matrix, and `quadratic_form` validates gradient dimension before computing `gᵀΣg`.

The covariance is not regularized, factorized, or required to be positive semidefinite. A symmetric supplied matrix can still be unsuitable for a probabilistic interpretation.
