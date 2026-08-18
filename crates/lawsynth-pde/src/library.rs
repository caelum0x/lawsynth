//! The differential-term candidate library.
//!
//! Following PDE-FIND, each candidate column is a product of a **power of the
//! field** and a **spatial-derivative factor**:
//!
//! ```text
//! column(p, m) = uᵖ · D_m,      D_0 = 1,  D_1 = u_x,  D_2 = u_xx,  D_3 = u_xxx
//! ```
//!
//! with `p` in `0..=max_u_degree` and `m` in `0..=max_derivative_order`. The
//! `(p = 0, m = 0)` column is the constant `1` (kept only when the config asks
//! for an intercept). This fixed cross-product covers the heat (`u_xx`), Burgers
//! (`u_xx`, `u·u_x`) and advection (`u_x`) families.
//!
//! The column order is deterministic: derivative order `m` is the outer loop and
//! field power `p` the inner loop, so the coefficient vector returned by the
//! sparse solve lines up with [`build_terms`] element-for-element.

use crate::PdeConfig;

/// One candidate column: the field raised to `u_power` times the
/// `derivative_order`-th spatial derivative (order `0` meaning the constant `1`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LibraryTerm {
    pub(crate) u_power: usize,
    pub(crate) derivative_order: usize,
    pub(crate) label: String,
}

/// Builds the ordered candidate library for a configuration.
///
/// The `(p = 0, m = 0)` constant column is included iff `config.include_constant`.
pub(crate) fn build_terms(config: &PdeConfig) -> Vec<LibraryTerm> {
    let variable = config.variable.as_str();
    let mut terms = Vec::new();
    for derivative_order in 0..=config.max_derivative_order {
        for u_power in 0..=config.max_u_degree {
            if derivative_order == 0 && u_power == 0 && !config.include_constant {
                continue;
            }
            terms.push(LibraryTerm {
                u_power,
                derivative_order,
                label: term_label(variable, u_power, derivative_order),
            });
        }
    }
    terms
}

/// Renders a human-readable label such as `1`, `u`, `u^2`, `u_x`, `u*u_x`,
/// `u^2*u_xx`.
pub(crate) fn term_label(variable: &str, u_power: usize, derivative_order: usize) -> String {
    let power_factor = match u_power {
        0 => None,
        1 => Some(variable.to_owned()),
        p => Some(format!("{variable}^{p}")),
    };
    let derivative_factor = match derivative_order {
        0 => None,
        m => Some(format!("{variable}_{}", "x".repeat(m))),
    };
    match (power_factor, derivative_factor) {
        (None, None) => "1".to_owned(),
        (Some(power), None) => power,
        (None, Some(derivative)) => derivative,
        (Some(power), Some(derivative)) => format!("{power}*{derivative}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_cover_the_expected_forms() {
        assert_eq!(term_label("u", 0, 0), "1");
        assert_eq!(term_label("u", 1, 0), "u");
        assert_eq!(term_label("u", 2, 0), "u^2");
        assert_eq!(term_label("u", 0, 1), "u_x");
        assert_eq!(term_label("u", 0, 2), "u_xx");
        assert_eq!(term_label("u", 0, 3), "u_xxx");
        assert_eq!(term_label("u", 1, 1), "u*u_x");
        assert_eq!(term_label("u", 2, 2), "u^2*u_xx");
    }

    #[test]
    fn default_library_has_the_pde_find_columns() {
        let config = PdeConfig::default();
        let terms = build_terms(&config);
        // degree 2 (u^0..u^2) × order 2 (D_0..D_2) = 9 columns.
        assert_eq!(terms.len(), 9);
        let labels: Vec<&str> = terms.iter().map(|t| t.label.as_str()).collect();
        assert_eq!(
            labels,
            vec!["1", "u", "u^2", "u_x", "u*u_x", "u^2*u_x", "u_xx", "u*u_xx", "u^2*u_xx"]
        );
    }

    #[test]
    fn dropping_the_constant_removes_only_the_intercept() {
        let config = PdeConfig::default().with_constant(false);
        let terms = build_terms(&config);
        assert_eq!(terms.len(), 8);
        assert!(terms.iter().all(|t| t.label != "1"));
    }

    #[test]
    fn ordering_is_derivative_outer_power_inner() {
        let config = PdeConfig::default();
        let terms = build_terms(&config);
        // The first `max_u_degree + 1` columns all share derivative order 0.
        assert!(terms[0..3].iter().all(|t| t.derivative_order == 0));
        assert!(terms[3..6].iter().all(|t| t.derivative_order == 1));
    }
}
