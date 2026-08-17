//! Exact / SVD Dynamic Mode Decomposition.

use crate::{Complex, Eigen, KoopmanError, Matrix, eigen, svd};

/// A fitted DMD operator with its spectral decomposition.
#[derive(Clone, Debug)]
pub struct DmdModel {
    operator: Matrix,
    reduced_operator: Matrix,
    eigenvalues: Vec<Complex>,
    modes: Vec<Vec<Complex>>,
    singular_values: Vec<f64>,
    rank: usize,
}

impl DmdModel {
    /// The identified full-state operator `A` such that `x' ≈ A x`.
    pub fn operator(&self) -> &Matrix {
        &self.operator
    }

    /// The `r × r` reduced operator `Ã` acting in the POD subspace.
    pub fn reduced_operator(&self) -> &Matrix {
        &self.reduced_operator
    }

    /// The discrete-time DMD eigenvalues (per step).
    pub fn eigenvalues(&self) -> &[Complex] {
        &self.eigenvalues
    }

    /// The exact DMD modes; `modes()[i]` is a length-`n` complex vector.
    pub fn modes(&self) -> &[Vec<Complex>] {
        &self.modes
    }

    /// The full singular-value spectrum of `X` (for effective-rank judgement).
    pub fn singular_values(&self) -> &[f64] {
        &self.singular_values
    }

    /// The truncation rank used to build the operator.
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// Continuous-time eigenvalues `ln(λ) / dt` (growth in `re`, angular
    /// frequency in `im`). Uses the principal branch of the logarithm.
    pub fn continuous_eigenvalues(&self, dt: f64) -> Vec<Complex> {
        self.eigenvalues.iter().map(|&lambda| lambda.ln().scale(1.0 / dt)).collect()
    }

    /// Rolls the operator forward `steps` times from `x0`, returning the states
    /// `[A·x0, A²·x0, …, A^steps·x0]` (not including `x0`).
    pub fn predict(&self, x0: &[f64], steps: usize) -> Result<Vec<Vec<f64>>, KoopmanError> {
        if x0.len() != self.operator.rows() {
            return Err(KoopmanError::ShapeMismatch);
        }
        let mut state = x0.to_vec();
        let mut trajectory = Vec::with_capacity(steps);
        for _ in 0..steps {
            state = self.operator.mat_vec(&state)?;
            trajectory.push(state.clone());
        }
        Ok(trajectory)
    }
}

/// Fits an exact/SVD DMD operator from aligned snapshot matrices.
///
/// `x` and `x_prime` share shape `n × m`: each column is a state observation and
/// `x_prime`'s columns are the one-step successors of `x`'s. `rank` truncates the
/// SVD of `x` and must lie in `1..=min(n, m)`.
pub fn dmd(x: &Matrix, x_prime: &Matrix, rank: usize) -> Result<DmdModel, KoopmanError> {
    if x.rows() != x_prime.rows() || x.cols() != x_prime.cols() {
        return Err(KoopmanError::ShapeMismatch);
    }
    let n = x.rows();
    let m = x.cols();
    let max_rank = n.min(m);
    if rank == 0 || rank > max_rank {
        return Err(KoopmanError::InvalidRank);
    }

    let decomposition = svd(x)?;
    let singular_values = decomposition.s.clone();

    let u_r = decomposition.u.first_columns(rank);
    // Build V_r · Σ_r⁻¹ by scaling each retained right-singular column.
    let mut v_scaled = decomposition.v.first_columns(rank);
    for (column, &sigma) in singular_values.iter().enumerate().take(rank) {
        if sigma == 0.0 {
            return Err(KoopmanError::InvalidRank);
        }
        v_scaled.scale_column(column, 1.0 / sigma);
    }

    // phi_base = X' · V_r · Σ_r⁻¹   (n × r)
    let phi_base = x_prime.matmul(&v_scaled)?;
    // A = phi_base · U_rᵀ           (n × n)
    let operator = phi_base.matmul(&u_r.transpose())?;
    // Ã = U_rᵀ · phi_base           (r × r)
    let reduced_operator = u_r.transpose().matmul(&phi_base)?;

    let Eigen { values, vectors } = eigen(&reduced_operator)?;
    let modes = exact_modes(&phi_base, &values, &vectors);

    Ok(DmdModel { operator, reduced_operator, eigenvalues: values, modes, singular_values, rank })
}

/// Exact DMD modes `φᵢ = (1/λᵢ) · phi_base · wᵢ` (Tu et al., 2014).
fn exact_modes(
    phi_base: &Matrix,
    eigenvalues: &[Complex],
    eigenvectors: &[Vec<Complex>],
) -> Vec<Vec<Complex>> {
    let n = phi_base.rows();
    eigenvalues
        .iter()
        .zip(eigenvectors)
        .map(|(&lambda, w)| {
            let mut mode = vec![Complex::ZERO; n];
            for (row, mode_entry) in mode.iter_mut().enumerate() {
                let mut sum = Complex::ZERO;
                for (column, &weight) in w.iter().enumerate() {
                    sum = sum.add(Complex::real(phi_base.get(row, column)).mul(weight));
                }
                *mode_entry = sum;
            }
            if !lambda.is_zero() {
                for entry in &mut mode {
                    *entry = entry.div(lambda);
                }
            }
            mode
        })
        .collect()
}
