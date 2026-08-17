//! DMD with control (DMDc): recover `x' ≈ A x + B u`.

use crate::{Complex, Eigen, KoopmanError, Matrix, eigen, svd};

/// A fitted controlled linear operator pair `(A, B)`.
#[derive(Clone, Debug)]
pub struct DmdcModel {
    a: Matrix,
    b: Matrix,
    singular_values: Vec<f64>,
    rank: usize,
}

impl DmdcModel {
    /// The state operator `A` (`n × n`).
    pub fn state_operator(&self) -> &Matrix {
        &self.a
    }

    /// The control operator `B` (`n × c`).
    pub fn control_operator(&self) -> &Matrix {
        &self.b
    }

    /// The singular-value spectrum of the stacked `[X; U]` matrix.
    pub fn singular_values(&self) -> &[f64] {
        &self.singular_values
    }

    /// The truncation rank used to build the operators.
    pub fn rank(&self) -> usize {
        self.rank
    }

    /// Eigenvalues of the recovered state operator `A`.
    pub fn state_eigenvalues(&self) -> Result<Vec<Complex>, KoopmanError> {
        let Eigen { values, .. } = eigen(&self.a)?;
        Ok(values)
    }

    /// Rolls `x_{t+1} = A x_t + B u_t` forward over the supplied `controls`,
    /// returning the states `[x_1, …, x_T]` where `T = controls.len()`.
    pub fn predict(
        &self,
        x0: &[f64],
        controls: &[Vec<f64>],
    ) -> Result<Vec<Vec<f64>>, KoopmanError> {
        if x0.len() != self.a.rows() {
            return Err(KoopmanError::ShapeMismatch);
        }
        let mut state = x0.to_vec();
        let mut trajectory = Vec::with_capacity(controls.len());
        for control in controls {
            if control.len() != self.b.cols() {
                return Err(KoopmanError::ShapeMismatch);
            }
            let drift = self.a.mat_vec(&state)?;
            let forced = self.b.mat_vec(control)?;
            state = drift.iter().zip(&forced).map(|(a, b)| a + b).collect();
            trajectory.push(state.clone());
        }
        Ok(trajectory)
    }
}

/// Fits `[A B]` from `x' = A x + B u` via a truncated SVD pseudo-inverse of the
/// stacked snapshot/control matrix `[X; U]`.
///
/// `x` and `x_prime` are `n × m`; `u` is `c × m` and column-aligned with them.
/// `rank` truncates the SVD of the stacked matrix and must lie in
/// `1..=min(n + c, m)`.
pub fn dmdc(
    x: &Matrix,
    x_prime: &Matrix,
    u: &Matrix,
    rank: usize,
) -> Result<DmdcModel, KoopmanError> {
    if x.rows() != x_prime.rows() || x.cols() != x_prime.cols() {
        return Err(KoopmanError::ShapeMismatch);
    }
    if u.cols() != x.cols() {
        return Err(KoopmanError::ShapeMismatch);
    }
    let n = x.rows();
    let c = u.rows();
    let m = x.cols();
    let stacked_rows = n + c;
    let max_rank = stacked_rows.min(m);
    if rank == 0 || rank > max_rank {
        return Err(KoopmanError::InvalidRank);
    }

    // Ω = [X; U]  (stacked (n + c) × m).
    let mut omega = Matrix::zeros(stacked_rows, m);
    for row in 0..n {
        for col in 0..m {
            omega.set(row, col, x.get(row, col));
        }
    }
    for row in 0..c {
        for col in 0..m {
            omega.set(n + row, col, u.get(row, col));
        }
    }

    let decomposition = svd(&omega)?;
    let singular_values = decomposition.s.clone();

    let u_r = decomposition.u.first_columns(rank);
    let mut v_scaled = decomposition.v.first_columns(rank);
    for (column, &sigma) in singular_values.iter().enumerate().take(rank) {
        if sigma == 0.0 {
            return Err(KoopmanError::InvalidRank);
        }
        v_scaled.scale_column(column, 1.0 / sigma);
    }

    // G = X' · V_r · Σ_r⁻¹ · U_rᵀ  = X' · pinv(Ω)   (n × (n + c)).
    let x_prime_v = x_prime.matmul(&v_scaled)?;
    let gain = x_prime_v.matmul(&u_r.transpose())?;

    // Split the gain into the state block A and control block B.
    let mut a = Matrix::zeros(n, n);
    let mut b = Matrix::zeros(n, c);
    for row in 0..n {
        for col in 0..n {
            a.set(row, col, gain.get(row, col));
        }
        for col in 0..c {
            b.set(row, col, gain.get(row, n + col));
        }
    }

    Ok(DmdcModel { a, b, singular_values, rank })
}
