//! A small, deterministic dense `f64` matrix in row-major storage.

use crate::KoopmanError;

/// A dense real matrix stored as `rows` vectors of length `cols`.
#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<Vec<f64>>,
}

impl Matrix {
    /// Builds a matrix from row-major data, validating rectangularity and
    /// finiteness.
    pub fn from_rows(data: Vec<Vec<f64>>) -> Result<Self, KoopmanError> {
        let rows = data.len();
        if rows == 0 {
            return Err(KoopmanError::EmptyMatrix);
        }
        let cols = data[0].len();
        if cols == 0 {
            return Err(KoopmanError::EmptyMatrix);
        }
        if data.iter().any(|row| row.len() != cols) {
            return Err(KoopmanError::ShapeMismatch);
        }
        if data.iter().flatten().any(|value| !value.is_finite()) {
            return Err(KoopmanError::NonFiniteValue);
        }
        Ok(Self { rows, cols, data })
    }

    /// A `rows × cols` matrix of zeros.
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self { rows, cols, data: vec![vec![0.0; cols]; rows] }
    }

    /// The `n × n` identity matrix.
    pub fn identity(n: usize) -> Self {
        let mut matrix = Self::zeros(n, n);
        for (index, row) in matrix.data.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        matrix
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row][col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row][col] = value;
    }

    /// Borrows the underlying row-major storage.
    pub fn data(&self) -> &[Vec<f64>] {
        &self.data
    }

    /// The transpose `Aᵀ`.
    pub fn transpose(&self) -> Self {
        let mut out = Self::zeros(self.cols, self.rows);
        for (row_index, row) in self.data.iter().enumerate() {
            for (col_index, &value) in row.iter().enumerate() {
                out.data[col_index][row_index] = value;
            }
        }
        out
    }

    /// The matrix product `self · other`.
    pub fn matmul(&self, other: &Self) -> Result<Self, KoopmanError> {
        if self.cols != other.rows {
            return Err(KoopmanError::ShapeMismatch);
        }
        let mut out = Self::zeros(self.rows, other.cols);
        for i in 0..self.rows {
            for k in 0..self.cols {
                let left = self.data[i][k];
                if left == 0.0 {
                    continue;
                }
                let source = &other.data[k];
                let target = &mut out.data[i];
                for j in 0..other.cols {
                    target[j] += left * source[j];
                }
            }
        }
        Ok(out)
    }

    /// The product `self · x` with a column vector.
    pub fn mat_vec(&self, x: &[f64]) -> Result<Vec<f64>, KoopmanError> {
        if x.len() != self.cols {
            return Err(KoopmanError::ShapeMismatch);
        }
        Ok(self.data.iter().map(|row| row.iter().zip(x).map(|(a, b)| a * b).sum()).collect())
    }

    /// The submatrix formed from the first `count` columns.
    pub fn first_columns(&self, count: usize) -> Self {
        let mut out = Self::zeros(self.rows, count);
        for (row_index, row) in self.data.iter().enumerate() {
            out.data[row_index][..count].copy_from_slice(&row[..count]);
        }
        out
    }

    /// Scales column `col` in place by `factor`.
    pub fn scale_column(&mut self, col: usize, factor: f64) {
        for row in &mut self.data {
            row[col] *= factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_ragged_and_non_finite_input() {
        assert_eq!(
            Matrix::from_rows(vec![vec![1.0, 2.0], vec![3.0]]).unwrap_err(),
            KoopmanError::ShapeMismatch
        );
        assert_eq!(
            Matrix::from_rows(vec![vec![1.0, f64::NAN]]).unwrap_err(),
            KoopmanError::NonFiniteValue
        );
        assert_eq!(Matrix::from_rows(vec![]).unwrap_err(), KoopmanError::EmptyMatrix);
    }

    #[test]
    fn multiplies_and_transposes() {
        let a = Matrix::from_rows(vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]]).unwrap();
        let b = Matrix::from_rows(vec![vec![1.0], vec![0.0], vec![-1.0]]).unwrap();
        let product = a.matmul(&b).unwrap();
        assert_eq!(product.get(0, 0), -2.0);
        assert_eq!(product.get(1, 0), -2.0);
        let transposed = a.transpose();
        assert_eq!(transposed.rows(), 3);
        assert_eq!(transposed.get(2, 1), 6.0);
    }

    #[test]
    fn identity_and_mat_vec() {
        let identity = Matrix::identity(3);
        let x = vec![7.0, -1.0, 2.0];
        assert_eq!(identity.mat_vec(&x).unwrap(), x);
    }

    #[test]
    fn shape_mismatch_is_reported() {
        let a = Matrix::identity(2);
        let b = Matrix::identity(3);
        assert_eq!(a.matmul(&b).unwrap_err(), KoopmanError::ShapeMismatch);
        assert_eq!(a.mat_vec(&[1.0]).unwrap_err(), KoopmanError::ShapeMismatch);
    }
}
