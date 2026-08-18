use crate::quadrature::trapezoid;
use crate::test_function::TestFunction;

/// The weak linear system assembled from data integrals against test functions.
///
/// For test function `φ_k`, candidate term `θ_c`, and state `x_i`:
///
/// - `library[k][c] = ∫ φ_k(t) · θ_c(x(t)) dt`
/// - `targets[i][k] = -∫ φ̇_k(t) · x_i(t) dt`
///
/// The target is the integration-by-parts form of `∫ φ_k ẋ_i dt`; because the
/// bump and its derivative vanish at the support boundary the boundary term is
/// exactly zero, so `x_i` is never differentiated. Solving `library · Ξ_i =
/// targets[i]` per state recovers the same coefficient matrix `Ξ` as strong-form
/// SINDy, without touching an estimated derivative of the data.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakSystem {
    /// `G`: one row per test function, one column per candidate term.
    pub library: Vec<Vec<f64>>,
    /// `B`: one target vector per state, each of length `K`.
    pub targets: Vec<Vec<f64>>,
}

/// Builds the weak system from the candidate design matrix and raw states.
///
/// `feature_rows[t][c] = θ_c(x(t))` is the candidate library evaluated on the
/// (observed, possibly noisy) states at each sample; `states[i]` is the raw
/// trajectory of state `i`. Integration uses the fixed trapezoidal quadrature
/// over the sample grid, so assembly is deterministic.
pub(crate) fn assemble(
    time: &[f64],
    feature_rows: &[Vec<f64>],
    states: &[&[f64]],
    tests: &[TestFunction],
) -> WeakSystem {
    let term_count = feature_rows.first().map_or(0, Vec::len);
    let sample_count = time.len();

    let mut library = Vec::with_capacity(tests.len());
    let mut targets = vec![Vec::with_capacity(tests.len()); states.len()];

    let mut phi = vec![0.0; sample_count];
    let mut phi_dot = vec![0.0; sample_count];
    let mut integrand = vec![0.0; sample_count];

    for test in tests {
        for (sample, &t) in time.iter().enumerate() {
            phi[sample] = test.value(t);
            phi_dot[sample] = test.derivative(t);
        }

        // Weak library row: ∫ φ_k · θ_c dt for every candidate column, formed by
        // accumulating the trapezoid contributions of each interval across all
        // columns at once in a single fixed left-to-right order.
        let mut row = vec![0.0; term_count];
        for sample in 0..sample_count - 1 {
            let dt = time[sample + 1] - time[sample];
            let left = 0.5 * dt * phi[sample];
            let right = 0.5 * dt * phi[sample + 1];
            let feat_left = &feature_rows[sample];
            let feat_right = &feature_rows[sample + 1];
            for (column, cell) in row.iter_mut().enumerate() {
                *cell += left * feat_left[column] + right * feat_right[column];
            }
        }
        library.push(row);

        // Weak target per state: -∫ φ̇_k · x_i dt (integration by parts).
        for (state_index, state) in states.iter().enumerate() {
            for (sample, cell) in integrand.iter_mut().enumerate() {
                *cell = phi_dot[sample] * state[sample];
            }
            targets[state_index].push(-trapezoid(time, &integrand));
        }
    }

    WeakSystem { library, targets }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_a_known_derivative_integral() {
        // x(t) = t on [0, 10]; ẋ = 1. With the constant candidate term θ = 1,
        // the weak identity is ∫ φ·1 dt = -∫ φ̇·t dt, so library[k][0] must
        // equal targets[0][k] for every test function.
        let time: Vec<f64> = (0..=1000).map(|i| i as f64 * 0.01).collect();
        let x: Vec<f64> = time.clone();
        let feature_rows: Vec<Vec<f64>> = time.iter().map(|_| vec![1.0]).collect();
        let tests = crate::test_function::place(&time, 4, 0.3, 4).unwrap();
        let system = assemble(&time, &feature_rows, &[x.as_slice()], &tests);
        for (row, target) in system.library.iter().zip(&system.targets[0]) {
            assert!((row[0] - target).abs() < 1e-6, "row {row:?} vs {target}");
        }
    }
}
