use crate::DifferentiationError;

/// Local quadratic Savitzky-Golay derivative estimate for arbitrary time axes.
pub fn savgol_series(
    time: &[f64],
    values: &[f64],
    window: usize,
) -> Result<Vec<f64>, DifferentiationError> {
    if time.len() != values.len() {
        return Err(DifferentiationError::LengthMismatch);
    }
    if window < 3 || window % 2 == 0 || window > time.len() {
        return Err(DifferentiationError::InvalidWindow);
    }
    let radius = window / 2;
    (0..time.len())
        .map(|index| {
            let start = index.saturating_sub(radius).min(time.len() - window);
            let end = start + window;
            let center = time[index];
            let mut normal = [[0.0; 3]; 3];
            let mut target = [0.0; 3];
            for point in start..end {
                let offset = time[point] - center;
                let basis = [1.0, offset, offset * offset];
                for row in 0..3 {
                    target[row] += basis[row] * values[point];
                    for column in 0..3 {
                        normal[row][column] += basis[row] * basis[column];
                    }
                }
            }
            solve_3x3(normal, target).map(|coefficients| coefficients[1])
        })
        .collect()
}

fn solve_3x3(
    mut matrix: [[f64; 3]; 3],
    mut target: [f64; 3],
) -> Result<[f64; 3], DifferentiationError> {
    for pivot in 0..3 {
        let best = (pivot..3)
            .max_by(|left, right| {
                matrix[*left][pivot]
                    .abs()
                    .total_cmp(&matrix[*right][pivot].abs())
            })
            .expect("three by three system has a pivot");
        if matrix[best][pivot].abs() < 1e-14 {
            return Err(DifferentiationError::SingularFit);
        }
        matrix.swap(pivot, best);
        target.swap(pivot, best);
        let scale = matrix[pivot][pivot];
        for value in matrix[pivot].iter_mut().skip(pivot) {
            *value /= scale;
        }
        target[pivot] /= scale;
        let pivot_row = matrix[pivot];
        for row in 0..3 {
            if row == pivot {
                continue;
            }
            let factor = matrix[row][pivot];
            for (column, value) in matrix[row].iter_mut().enumerate().skip(pivot) {
                *value -= factor * pivot_row[column];
            }
            target[row] -= factor * target[pivot];
        }
    }
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_quadratic_derivatives_on_a_regular_grid() {
        let time = (0..11).map(|value| value as f64).collect::<Vec<_>>();
        let values = time.iter().map(|value| value * value).collect::<Vec<_>>();
        let derivative = savgol_series(&time, &values, 5).unwrap();
        assert!(
            derivative
                .iter()
                .zip(time)
                .all(|(actual, time)| (actual - 2.0 * time).abs() < 1e-10)
        );
    }
}
