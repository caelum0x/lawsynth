use crate::UncertaintyError;

/// A checked, row-major symmetric covariance matrix.
#[derive(Clone, Debug, PartialEq)]
pub struct CovarianceMatrix {
    dimension: usize,
    data: Vec<f64>,
}

impl CovarianceMatrix {
    /// Estimates an unbiased covariance matrix from row-major observations.
    pub fn from_observations(rows: &[Vec<f64>]) -> Result<Self, UncertaintyError> {
        if rows.len() < 2 {
            return Err(UncertaintyError::TooFewSamples {
                minimum: 2,
                actual: rows.len(),
            });
        }
        let dimension = rows.first().map_or(0, Vec::len);
        if dimension == 0 {
            return Err(UncertaintyError::EmptyInput);
        }
        for row in rows {
            if row.len() != dimension {
                return Err(UncertaintyError::DimensionMismatch {
                    expected: dimension,
                    actual: row.len(),
                });
            }
            if row.iter().any(|value| !value.is_finite()) {
                return Err(UncertaintyError::NonFiniteValue);
            }
        }
        let mut means = vec![0.0; dimension];
        for row in rows {
            for (column, value) in row.iter().enumerate() {
                means[column] += value;
            }
        }
        for mean in &mut means {
            *mean /= rows.len() as f64;
        }
        let mut data = vec![0.0; dimension * dimension];
        for row in rows {
            for i in 0..dimension {
                for j in 0..dimension {
                    data[i * dimension + j] += (row[i] - means[i]) * (row[j] - means[j]);
                }
            }
        }
        for value in &mut data {
            *value /= (rows.len() - 1) as f64;
        }
        Ok(Self { dimension, data })
    }

    pub fn from_row_major(dimension: usize, data: Vec<f64>) -> Result<Self, UncertaintyError> {
        if dimension == 0 || data.len() != dimension * dimension {
            return Err(UncertaintyError::DimensionMismatch {
                expected: dimension * dimension,
                actual: data.len(),
            });
        }
        if data.iter().any(|value| !value.is_finite()) {
            return Err(UncertaintyError::NonFiniteValue);
        }
        for i in 0..dimension {
            for j in 0..dimension {
                if (data[i * dimension + j] - data[j * dimension + i]).abs() > 1e-12 {
                    return Err(UncertaintyError::DimensionMismatch {
                        expected: dimension,
                        actual: dimension,
                    });
                }
            }
        }
        Ok(Self { dimension, data })
    }
    pub fn dimension(&self) -> usize {
        self.dimension
    }
    pub fn get(&self, row: usize, column: usize) -> Option<f64> {
        (row < self.dimension && column < self.dimension)
            .then(|| self.data[row * self.dimension + column])
    }
    pub fn as_row_major(&self) -> &[f64] {
        &self.data
    }
    pub fn quadratic_form(&self, gradient: &[f64]) -> Result<f64, UncertaintyError> {
        if gradient.len() != self.dimension {
            return Err(UncertaintyError::DimensionMismatch {
                expected: self.dimension,
                actual: gradient.len(),
            });
        }
        if gradient.iter().any(|value| !value.is_finite()) {
            return Err(UncertaintyError::NonFiniteValue);
        }
        Ok((0..self.dimension)
            .flat_map(|i| {
                (0..self.dimension)
                    .map(move |j| gradient[i] * self.data[i * self.dimension + j] * gradient[j])
            })
            .sum())
    }
}
