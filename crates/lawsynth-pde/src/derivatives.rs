//! Second-order-accurate central finite-difference stencils on the interior of a
//! regular grid.
//!
//! All stencils below are the standard centred differences with truncation error
//! `O(h²)`. They are only valid on the *interior* of the grid — a stencil of
//! half-width `h` may not be evaluated within `h` points of a boundary — so the
//! discovery pipeline drops the outermost points along each axis.
//!
//! | quantity | stencil | half-width | order |
//! |----------|---------|-----------|-------|
//! | `u_x`    | `(u[i+1] − u[i−1]) / (2·dx)`                         | 1 | O(dx²) |
//! | `u_xx`   | `(u[i+1] − 2·u[i] + u[i−1]) / dx²`                   | 1 | O(dx²) |
//! | `u_xxx`  | `(u[i+2] − 2·u[i+1] + 2·u[i−1] − u[i−2]) / (2·dx³)`  | 2 | O(dx²) |
//! | `u_t`    | `(u[t+1] − u[t−1]) / (2·dt)`                         | 1 | O(dt²) |

/// The half-width (number of neighbours needed on each side) of the central
/// stencil for a spatial derivative of the given `order`.
///
/// `order == 0` (the field itself) needs no neighbours; orders `1` and `2` need
/// one; order `3` needs two. The closed form `(order + 1) / 2` reproduces this.
pub(crate) const fn spatial_half_width(order: usize) -> usize {
    order.div_ceil(2)
}

/// Evaluates the `order`-th central spatial derivative of a single snapshot
/// `row` at interior index `x`.
///
/// The caller MUST guarantee `spatial_half_width(order) <= x` and
/// `x + spatial_half_width(order) < row.len()`; the interior loop in
/// [`crate::discover_pde`] enforces exactly this. `order == 0` is not a
/// derivative — the library treats the zeroth "derivative factor" as the
/// constant `1`, so this function only serves `order >= 1`.
pub(crate) fn spatial_derivative(row: &[f64], x: usize, order: usize, dx: f64) -> f64 {
    match order {
        1 => (row[x + 1] - row[x - 1]) / (2.0 * dx),
        2 => (row[x + 1] - 2.0 * row[x] + row[x - 1]) / (dx * dx),
        3 => (row[x + 2] - 2.0 * row[x + 1] + 2.0 * row[x - 1] - row[x - 2]) / (2.0 * dx * dx * dx),
        other => panic!("unsupported spatial derivative order {other}; config caps it at 3"),
    }
}

/// The central time derivative `u_t` at interior time `t`, spatial index `x`.
///
/// The caller MUST guarantee `1 <= t` and `t + 1 < field.len()`.
pub(crate) fn time_derivative(field: &[Vec<f64>], t: usize, x: usize, dt: f64) -> f64 {
    (field[t + 1][x] - field[t - 1][x]) / (2.0 * dt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn half_widths_match_the_stencils() {
        assert_eq!(spatial_half_width(0), 0);
        assert_eq!(spatial_half_width(1), 1);
        assert_eq!(spatial_half_width(2), 1);
        assert_eq!(spatial_half_width(3), 2);
    }

    #[test]
    fn first_derivative_is_exact_for_a_linear_field() {
        // f(x) = 3x + 1 sampled at spacing 0.5 → f'(x) = 3 exactly (no truncation).
        let dx = 0.5;
        let row: Vec<f64> = (0..5).map(|i| 3.0 * (i as f64 * dx) + 1.0).collect();
        for x in 1..row.len() - 1 {
            assert!((spatial_derivative(&row, x, 1, dx) - 3.0).abs() < 1e-12);
        }
    }

    #[test]
    fn second_derivative_is_exact_for_a_quadratic_field() {
        // f(x) = 2x² → f''(x) = 4 exactly for the centred second difference.
        let dx = 0.25;
        let row: Vec<f64> = (0..7).map(|i| 2.0 * (i as f64 * dx).powi(2)).collect();
        for x in 1..row.len() - 1 {
            assert!((spatial_derivative(&row, x, 2, dx) - 4.0).abs() < 1e-10);
        }
    }

    #[test]
    fn third_derivative_is_exact_for_a_cubic_field() {
        // f(x) = x³ → f'''(x) = 6 exactly for the centred third difference.
        let dx = 0.3;
        let row: Vec<f64> = (0..9).map(|i| (i as f64 * dx).powi(3)).collect();
        for x in 2..row.len() - 2 {
            assert!((spatial_derivative(&row, x, 3, dx) - 6.0).abs() < 1e-8);
        }
    }

    #[test]
    fn first_derivative_recovers_a_cosine_within_truncation() {
        // f(x) = sin(x): f'(x) = cos(x). Central diff is O(dx²)-accurate.
        let dx = 0.01;
        let row: Vec<f64> = (0..64).map(|i| (i as f64 * dx).sin()).collect();
        for x in 1..row.len() - 1 {
            let expected = (x as f64 * dx).cos();
            assert!((spatial_derivative(&row, x, 1, dx) - expected).abs() < 1e-4);
        }
    }

    #[test]
    fn time_derivative_is_exact_for_a_linear_ramp() {
        // u(t, x) = 5t → u_t = 5 exactly.
        let dt = 0.2;
        let field: Vec<Vec<f64>> = (0..4).map(|t| vec![5.0 * t as f64 * dt; 3]).collect();
        for t in 1..field.len() - 1 {
            for x in 0..3 {
                assert!((time_derivative(&field, t, x, dt) - 5.0).abs() < 1e-12);
            }
        }
    }
}
