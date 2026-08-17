//! Deterministic Koopman / DMD linear-operator discovery.
//!
//! This crate discovers a linear operator that advances a system one step in
//! time from snapshot pairs: exact/SVD DMD (`x' ≈ A x`), DMD with control
//! (`x' ≈ A x + B u`), and Extended DMD over a feature dictionary
//! (`ψ(x') ≈ K ψ(x)`, the bridge to nonlinear dynamics). The linear algebra —
//! a one-sided Jacobi SVD and a Wilkinson-shifted complex QR eigensolver — is
//! hand-rolled with the standard library only, so every fit is deterministic
//! and offline.
//!
//! The discovered object is honestly a *linear* (or lifted-linear)
//! approximation, not a nonlinear symbolic law; see `specs/koopman/README.md`.

mod complex;
mod dmd;
mod dmdc;
mod edmd;
mod eig;
mod error;
mod matrix;
mod svd;

pub use complex::Complex;
pub use dmd::{DmdModel, dmd};
pub use dmdc::{DmdcModel, dmdc};
pub use edmd::{EdmdModel, PolynomialDictionary, edmd};
pub use eig::{Eigen, eigen};
pub use error::KoopmanError;
pub use matrix::Matrix;
pub use svd::{Svd, svd};

use lawsynth_data::Dataset;

/// Builds `(X, X')` snapshot matrices from a dataset's columns.
///
/// Each column of the returned matrices is a state observation: row `i` is the
/// `i`-th identifier-sorted dataset column, column `t` is time step `t`. `X`
/// spans steps `0..m-1` and `X'` spans steps `1..m`, so the two are aligned
/// one-step-shifted successors suitable for [`dmd`].
pub fn snapshots_from_dataset(dataset: &Dataset) -> Result<(Matrix, Matrix), KoopmanError> {
    let columns: Vec<&Vec<f64>> = dataset.columns().values().map(|column| &column.values).collect();
    let state_dim = columns.len();
    let time_len = dataset.time().len();
    if time_len < 2 {
        return Err(KoopmanError::InsufficientSnapshots);
    }
    let pairs = time_len - 1;
    let mut x = Matrix::zeros(state_dim, pairs);
    let mut x_prime = Matrix::zeros(state_dim, pairs);
    for (row, column) in columns.iter().enumerate() {
        for step in 0..pairs {
            x.set(row, step, column[step]);
            x_prime.set(row, step, column[step + 1]);
        }
    }
    Ok((x, x_prime))
}
