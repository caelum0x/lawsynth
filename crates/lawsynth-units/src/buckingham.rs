//! Buckingham-π: dimensionless groups from a set of variable dimensions.
//!
//! Given `n` variables with SI dimension vectors (integer exponents over the 7
//! base dimensions), a *dimensionless group* is an integer exponent vector `p`
//! with `D · p = 0`, where `D` is the `7 × n` dimension matrix. The set of such
//! `p` is the integer nullspace of `D`; a basis of it is a maximal independent
//! family of dimensionless monomials `∏ xⱼ^{pⱼ}`.
//!
//! The computation is exact (rational Gaussian elimination over `i128`
//! fractions) and deterministic: the basis is normalized to primitive integer
//! vectors with a positive leading entry and returned in sorted order. This is a
//! standard-library-only implementation — no external linear-algebra crate.

use crate::Dimension;

const BASE_DIMENSIONS: usize = 7;

/// Returns a basis of the dimensionless groups for `dimensions`.
///
/// Each returned vector has the same length as `dimensions` and lists the
/// integer exponents of a dimensionless monomial in input order. The basis is
/// empty when the variables admit no non-trivial dimensionless combination (the
/// dimension matrix has full column rank).
pub fn dimensionless_groups(dimensions: &[Dimension]) -> Vec<Vec<i64>> {
    let columns = dimensions.len();
    if columns == 0 {
        return Vec::new();
    }

    // Rational row-reduce the 7 x n dimension matrix.
    let mut matrix: Vec<Vec<Rational>> = (0..BASE_DIMENSIONS)
        .map(|row| {
            dimensions
                .iter()
                .map(|dimension| Rational::from_integer(i64::from(dimension.exponents()[row])))
                .collect()
        })
        .collect();

    let pivot_columns = reduced_row_echelon(&mut matrix, columns);
    let is_pivot = {
        let mut flags = vec![false; columns];
        for &column in &pivot_columns {
            flags[column] = true;
        }
        flags
    };

    // One nullspace basis vector per free column.
    let mut basis: Vec<Vec<i64>> = Vec::new();
    for free_column in (0..columns).filter(|column| !is_pivot[*column]) {
        let mut vector = vec![Rational::from_integer(0); columns];
        vector[free_column] = Rational::from_integer(1);
        for (pivot_row, &pivot_column) in pivot_columns.iter().enumerate() {
            // pivot value is 1 after RREF, so pⱼ = -matrix[row][free_column].
            vector[pivot_column] = matrix[pivot_row][free_column].negated();
        }
        basis.push(normalize_to_integers(&vector));
    }

    basis.sort();
    basis
}

/// Reduces `matrix` (with `columns` columns) to reduced row echelon form in
/// place and returns the pivot column of each nonzero pivot row, in row order.
fn reduced_row_echelon(matrix: &mut [Vec<Rational>], columns: usize) -> Vec<usize> {
    let mut pivot_columns = Vec::new();
    let mut pivot_row = 0;
    for column in 0..columns {
        let Some(selected) = (pivot_row..matrix.len()).find(|&row| !matrix[row][column].is_zero())
        else {
            continue;
        };
        matrix.swap(pivot_row, selected);
        let inverse = matrix[pivot_row][column].reciprocal();
        for entry in &mut matrix[pivot_row] {
            *entry = entry.multiply(inverse);
        }
        let pivot_values = matrix[pivot_row].clone();
        for (row, current) in matrix.iter_mut().enumerate() {
            if row != pivot_row && !current[column].is_zero() {
                let factor = current[column];
                for (target, pivot_value) in current.iter_mut().zip(&pivot_values) {
                    *target = target.subtract(factor.multiply(*pivot_value));
                }
            }
        }
        pivot_columns.push(column);
        pivot_row += 1;
        if pivot_row == matrix.len() {
            break;
        }
    }
    pivot_columns
}

/// Scales a rational vector to the primitive integer vector with the same
/// direction and a positive leading nonzero entry.
fn normalize_to_integers(vector: &[Rational]) -> Vec<i64> {
    let mut denominator_lcm: i64 = 1;
    for value in vector {
        denominator_lcm = lcm(denominator_lcm, value.denominator);
    }
    let mut integers: Vec<i64> = vector
        .iter()
        .map(|value| value.numerator * (denominator_lcm / value.denominator))
        .collect();

    let mut divisor: i64 = 0;
    for value in &integers {
        divisor = gcd(divisor, value.abs());
    }
    if divisor > 1 {
        for value in &mut integers {
            *value /= divisor;
        }
    }
    if let Some(leading) = integers.iter().copied().find(|value| *value != 0) {
        if leading < 0 {
            for value in &mut integers {
                *value = -*value;
            }
        }
    }
    integers
}

/// An exact rational over `i128`, always stored in lowest terms with a positive
/// denominator.
#[derive(Clone, Copy, Debug)]
struct Rational {
    numerator: i64,
    denominator: i64,
}

impl Rational {
    fn from_integer(value: i64) -> Self {
        Self { numerator: value, denominator: 1 }
    }

    fn new(numerator: i128, denominator: i128) -> Self {
        debug_assert!(denominator != 0, "rational denominator must be nonzero");
        let sign = if denominator < 0 { -1 } else { 1 };
        let numerator = numerator * sign;
        let denominator = denominator * sign;
        let divisor = gcd_i128(numerator.abs(), denominator.abs()).max(1);
        Self {
            numerator: (numerator / divisor) as i64,
            denominator: (denominator / divisor) as i64,
        }
    }

    fn is_zero(self) -> bool {
        self.numerator == 0
    }

    fn negated(self) -> Self {
        Self { numerator: -self.numerator, denominator: self.denominator }
    }

    fn reciprocal(self) -> Self {
        debug_assert!(self.numerator != 0, "cannot invert zero");
        Rational::new(i128::from(self.denominator), i128::from(self.numerator))
    }

    fn multiply(self, other: Self) -> Self {
        Rational::new(
            i128::from(self.numerator) * i128::from(other.numerator),
            i128::from(self.denominator) * i128::from(other.denominator),
        )
    }

    fn subtract(self, other: Self) -> Self {
        Rational::new(
            i128::from(self.numerator) * i128::from(other.denominator)
                - i128::from(other.numerator) * i128::from(self.denominator),
            i128::from(self.denominator) * i128::from(other.denominator),
        )
    }
}

fn gcd(a: i64, b: i64) -> i64 {
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

fn gcd_i128(a: i128, b: i128) -> i128 {
    let mut a = a.abs();
    let mut b = b.abs();
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

fn lcm(a: i64, b: i64) -> i64 {
    if a == 0 || b == 0 {
        return 1;
    }
    (a / gcd(a, b)) * b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Unit;

    fn dimension(expression: &str) -> Dimension {
        Unit::parse(expression).unwrap().dimension()
    }

    #[test]
    fn length_time_velocity_yield_one_group() {
        // x [m], t [s], v [m/s]  ->  the group v·t/x is dimensionless.
        let dimensions = [dimension("m"), dimension("s"), dimension("m/s")];
        let groups = dimensionless_groups(&dimensions);
        // The basis is normalized to a positive leading entry, so the group is
        // reported as x / (t·v) — the reciprocal of v·t/x, equally dimensionless.
        assert_eq!(groups, vec![vec![1, -1, -1]]);
    }

    #[test]
    fn every_group_is_actually_dimensionless() {
        let dimensions = [dimension("m"), dimension("s"), dimension("m/s")];
        for group in dimensionless_groups(&dimensions) {
            let mut product = Dimension::DIMENSIONLESS;
            for (variable, &exponent) in dimensions.iter().zip(&group) {
                product = product.multiply(variable.pow(exponent as i8).unwrap()).unwrap();
            }
            assert_eq!(product, Dimension::DIMENSIONLESS);
        }
    }

    #[test]
    fn independent_dimensions_have_no_group() {
        let dimensions = [dimension("m"), dimension("s"), dimension("kg")];
        assert!(dimensionless_groups(&dimensions).is_empty());
    }

    #[test]
    fn a_repeated_dimension_yields_a_ratio_group() {
        // Two lengths -> the ratio x2/x1 is dimensionless.
        let dimensions = [dimension("m"), dimension("m")];
        assert_eq!(dimensionless_groups(&dimensions), vec![vec![1, -1]]);
    }

    #[test]
    fn the_count_matches_the_buckingham_theorem() {
        // n = 4 variables spanning 3 independent base dimensions -> n - rank = 1.
        let dimensions = [dimension("m"), dimension("s"), dimension("kg"), dimension("km/min")];
        assert_eq!(dimensionless_groups(&dimensions).len(), 1);
    }

    #[test]
    fn empty_input_has_no_groups() {
        assert!(dimensionless_groups(&[]).is_empty());
    }
}
