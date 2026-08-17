//! Extended DMD (EDMD): fit a linear operator in a lifted observable space.
//!
//! EDMD lifts each state snapshot through a fixed feature dictionary `ψ` and
//! fits `ψ(x') ≈ K ψ(x)`. With a rich enough dictionary this linear operator on
//! observables approximates the nonlinear flow far better than raw DMD on the
//! bare state — this is the Koopman bridge to nonlinear dynamics. The dictionary
//! here is a deterministic total-degree polynomial basis; it is small and
//! self-contained rather than a dependency on the feature crate.

use lawsynth_data::Dataset;

use crate::{Complex, DmdModel, KoopmanError, Matrix, dmd};

/// A deterministic total-degree monomial dictionary over the state variables.
#[derive(Clone, Debug)]
pub struct PolynomialDictionary {
    variables: usize,
    degree: usize,
    exponents: Vec<Vec<usize>>,
}

impl PolynomialDictionary {
    /// Builds the monomial basis of total degree `≤ degree` over `variables`
    /// state coordinates. Features are ordered by total degree, then
    /// lexicographically, so index 0 is the constant term.
    pub fn new(variables: usize, degree: usize) -> Result<Self, KoopmanError> {
        if variables == 0 || degree == 0 {
            return Err(KoopmanError::InvalidDictionary);
        }
        let mut exponents = Vec::new();
        for total in 0..=degree {
            exponents.extend(tuples_with_sum(variables, total));
        }
        Ok(Self { variables, degree, exponents })
    }

    /// The number of state coordinates the dictionary expects.
    pub fn variables(&self) -> usize {
        self.variables
    }

    /// The maximum total polynomial degree.
    pub fn degree(&self) -> usize {
        self.degree
    }

    /// The number of lifted features `ψ` produces.
    pub fn feature_count(&self) -> usize {
        self.exponents.len()
    }

    /// Lifts a raw state vector into the observable space `ψ(x)`.
    pub fn lift(&self, state: &[f64]) -> Result<Vec<f64>, KoopmanError> {
        if state.len() != self.variables {
            return Err(KoopmanError::ShapeMismatch);
        }
        Ok(self
            .exponents
            .iter()
            .map(|exponent| {
                exponent
                    .iter()
                    .zip(state)
                    .map(|(&power, &value)| value.powi(power as i32))
                    .product()
            })
            .collect())
    }

    /// The lifted-feature index of each degree-one monomial `xᵢ`, used to read
    /// the raw state back out of a lifted vector.
    fn state_indices(&self) -> Vec<usize> {
        (0..self.variables)
            .map(|variable| {
                self.exponents
                    .iter()
                    .position(|exponent| {
                        exponent[variable] == 1
                            && exponent.iter().enumerate().all(|(k, &e)| k == variable || e == 0)
                    })
                    .expect("degree-one monomial is always present")
            })
            .collect()
    }
}

/// Enumerates every non-negative exponent tuple of length `variables` that sums
/// to exactly `sum`, in lexicographic order.
fn tuples_with_sum(variables: usize, sum: usize) -> Vec<Vec<usize>> {
    if variables == 1 {
        return vec![vec![sum]];
    }
    let mut out = Vec::new();
    for first in 0..=sum {
        for rest in tuples_with_sum(variables - 1, sum - first) {
            let mut tuple = Vec::with_capacity(variables);
            tuple.push(first);
            tuple.extend(rest);
            out.push(tuple);
        }
    }
    out
}

/// A fitted EDMD model: a DMD operator in the lifted space plus the dictionary.
#[derive(Clone, Debug)]
pub struct EdmdModel {
    dmd: DmdModel,
    dictionary: PolynomialDictionary,
    state_indices: Vec<usize>,
}

impl EdmdModel {
    /// The lifted-space Koopman operator `K`.
    pub fn koopman_operator(&self) -> &Matrix {
        self.dmd.operator()
    }

    /// The eigenvalues of the lifted operator (Koopman spectrum estimate).
    pub fn eigenvalues(&self) -> &[Complex] {
        self.dmd.eigenvalues()
    }

    /// The underlying DMD model in the lifted space.
    pub fn dmd_model(&self) -> &DmdModel {
        &self.dmd
    }

    /// The dictionary used to lift states.
    pub fn dictionary(&self) -> &PolynomialDictionary {
        &self.dictionary
    }

    /// Predicts `steps` future raw states from `x0` by re-lifting each step:
    /// `x_{t+1} = readout(K · ψ(x_t))`. Re-lifting keeps the observable vector
    /// on the monomial manifold, which is the standard EDMD forecasting mode.
    pub fn predict(&self, x0: &[f64], steps: usize) -> Result<Vec<Vec<f64>>, KoopmanError> {
        if x0.len() != self.dictionary.variables() {
            return Err(KoopmanError::ShapeMismatch);
        }
        let mut state = x0.to_vec();
        let mut trajectory = Vec::with_capacity(steps);
        for _ in 0..steps {
            let lifted = self.dictionary.lift(&state)?;
            let advanced = self.dmd.operator().mat_vec(&lifted)?;
            state = self.state_indices.iter().map(|&index| advanced[index]).collect();
            trajectory.push(state.clone());
        }
        Ok(trajectory)
    }
}

/// Fits an EDMD operator over a dataset using the supplied polynomial dictionary.
///
/// State snapshots are the dataset's columns (identifier-sorted) at each time
/// step; consecutive rows form the snapshot pairs. `rank` truncates the SVD in
/// the lifted space and must lie in `1..=min(feature_count, snapshots)`.
pub fn edmd(
    dataset: &Dataset,
    dictionary: &PolynomialDictionary,
    rank: usize,
) -> Result<EdmdModel, KoopmanError> {
    let columns: Vec<&Vec<f64>> = dataset.columns().values().map(|column| &column.values).collect();
    let state_dim = columns.len();
    if dictionary.variables() != state_dim {
        return Err(KoopmanError::InvalidDictionary);
    }
    let time_len = dataset.time().len();
    if time_len < 2 {
        return Err(KoopmanError::InsufficientSnapshots);
    }

    let features = dictionary.feature_count();
    let pairs = time_len - 1;
    let mut lifted_x = Matrix::zeros(features, pairs);
    let mut lifted_next = Matrix::zeros(features, pairs);
    for step in 0..pairs {
        let state: Vec<f64> = columns.iter().map(|column| column[step]).collect();
        let next: Vec<f64> = columns.iter().map(|column| column[step + 1]).collect();
        let lifted_state = dictionary.lift(&state)?;
        let lifted_successor = dictionary.lift(&next)?;
        for row in 0..features {
            lifted_x.set(row, step, lifted_state[row]);
            lifted_next.set(row, step, lifted_successor[row]);
        }
    }

    let model = dmd(&lifted_x, &lifted_next, rank)?;
    Ok(EdmdModel {
        dmd: model,
        dictionary: dictionary.clone(),
        state_indices: dictionary.state_indices(),
    })
}
