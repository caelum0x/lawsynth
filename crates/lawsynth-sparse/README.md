# lawsynth-sparse

Deterministic sparse-regression solvers for fitting candidate equation terms.
The crate accepts a validated dense `RegressionProblem` and offers STLSQ, SR3,
coordinate-descent LASSO, grouped thresholding, non-negative least squares,
feature scaling, and deterministic bootstrap stability selection.

## Use

```rust
use lawsynth_sparse::{RegressionProblem, SparseConfig, stlsq};

let problem = RegressionProblem::new(vec![vec![1.0], vec![2.0], vec![3.0]], vec![2.0, 4.0, 6.0])?;
let solution = stlsq(&problem, &SparseConfig::default())?;
assert_eq!(solution.coefficients.len(), 1);
# Ok::<(), lawsynth_sparse::SparseError>(())
```

Rows are observations and columns are features. Solvers reject ragged or
non-finite inputs. Sparse selection is a model-selection mechanism, not proof
that excluded terms have no physical or causal effect.
