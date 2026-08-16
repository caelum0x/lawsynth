use crate::SparseError;

/// A validated row-major design matrix and scalar target vector.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionProblem {
    pub rows: Vec<Vec<f64>>,
    pub target: Vec<f64>,
}

impl RegressionProblem {
    pub fn new(rows: Vec<Vec<f64>>, target: Vec<f64>) -> Result<Self, SparseError> {
        let Some(first) = rows.first() else {
            return Err(SparseError::EmptyProblem);
        };
        if first.is_empty() || rows.len() != target.len() {
            return Err(SparseError::RowLengthMismatch);
        }
        if rows.iter().any(|row| row.len() != first.len()) {
            return Err(SparseError::RowLengthMismatch);
        }
        if rows
            .iter()
            .flatten()
            .chain(&target)
            .any(|value| !value.is_finite())
        {
            return Err(SparseError::NonFiniteValue);
        }
        Ok(Self { rows, target })
    }

    pub fn features(&self) -> usize {
        self.rows[0].len()
    }
}
