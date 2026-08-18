use lawsynth_data::Dataset;
use lawsynth_features::FeatureLibrary;

use crate::assembly::assemble;
use crate::solve::{StlsqConfig, conditioning, stlsq};
use crate::test_function::place;
use crate::{WeakConfig, WeakError};

/// Minimum number of samples weak assembly requires.
const MINIMUM_SAMPLES: usize = 4;

/// A single non-zero term of a discovered weak-form law.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakTerm {
    /// Human-readable candidate term name (e.g. `"x"`, `"x*y"`).
    pub name: String,
    /// Fitted coefficient.
    pub coefficient: f64,
}

/// A discovered governing law `d/dt state = Σ coefficient · term`.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakLaw {
    /// The state variable whose time-derivative this law describes.
    pub state: String,
    /// The surviving (non-zero) terms, in candidate-library order.
    pub terms: Vec<WeakTerm>,
}

impl WeakLaw {
    /// Renders the law as `d/dt state = a*term + b*term …` for display.
    pub fn render(&self) -> String {
        if self.terms.is_empty() {
            return format!("d/dt {} = 0", self.state);
        }
        let body = self
            .terms
            .iter()
            .map(|term| format!("{:.6}*{}", term.coefficient, term.name))
            .collect::<Vec<_>>()
            .join(" + ");
        format!("d/dt {} = {body}", self.state)
    }
}

/// Diagnostics describing a weak-form solve.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakDiagnostics {
    /// Number of test functions `K` used.
    pub test_functions: usize,
    /// Number of candidate library terms `C`.
    pub library_terms: usize,
    /// Conditioning proxy of the weak library (see [`conditioning`]).
    pub condition: f64,
    /// Per-state weak residual `‖G Ξ_i − B_i‖₂`, in state order.
    pub residuals: Vec<f64>,
    /// Largest per-state residual.
    pub max_residual: f64,
}

/// The result of weak / integral-form discovery.
#[derive(Clone, Debug, PartialEq)]
pub struct WeakResult {
    /// One discovered law per state variable, in schema (lexicographic) order.
    pub laws: Vec<WeakLaw>,
    /// The full coefficient matrix `Ξ`: `coefficients[state][term]`.
    pub coefficients: Vec<Vec<f64>>,
    /// Candidate library term names, aligned with the columns of `Ξ`.
    pub term_names: Vec<String>,
    /// State variable names, aligned with the rows of `Ξ`.
    pub state_names: Vec<String>,
    /// Solve diagnostics.
    pub diagnostics: WeakDiagnostics,
}

/// Discovers governing dynamics from a dataset using the weak / integral form.
///
/// The observed data is **never differentiated**. For each state `x_i` and each
/// compactly-supported test function `φ_k`, the ODE `ẋ_i = Θ(x) Ξ_i` is
/// multiplied by `φ_k` and integrated over the subdomain; integration by parts
/// moves the derivative onto the analytic `φ`, yielding the linear system
/// `∫ φ̇_k x_i dt = −∫ φ_k Θ(x) dt · Ξ_i` (signs folded into the assembly). The
/// per-state over-determined systems are solved by sequentially-thresholded
/// least squares. Everything — test-function placement, quadrature, and the
/// solve — is deterministic, so identical inputs yield bit-identical output.
pub fn weak_discover(dataset: &Dataset, config: &WeakConfig) -> Result<WeakResult, WeakError> {
    config.validate()?;

    let time = dataset.time().values();
    if time.len() < MINIMUM_SAMPLES {
        return Err(WeakError::TooFewSamples { available: time.len(), required: MINIMUM_SAMPLES });
    }

    // State variables and their raw trajectories in schema (lexicographic) order.
    let variables = dataset.schema().columns;
    let state_names: Vec<String> =
        dataset.columns().values().map(|column| column.id.as_str().to_string()).collect();
    let states: Vec<&[f64]> =
        dataset.columns().values().map(|column| column.values.as_slice()).collect();

    // Candidate library evaluated on the observed states: feature_rows[t][c].
    let library =
        FeatureLibrary::polynomial(variables, config.feature_degree, config.include_constant)
            .map_err(|error| WeakError::Feature(error.to_string()))?;
    let matrix =
        library.evaluate(dataset).map_err(|error| WeakError::Feature(error.to_string()))?;
    let term_names: Vec<String> = matrix.terms.iter().map(|term| term.name.clone()).collect();

    // Deterministic test-function bank and weak system assembly.
    let tests = place(
        time,
        config.test_function_count,
        config.support_fraction,
        config.test_function_order,
    )?;
    let system = assemble(time, &matrix.rows, &states, &tests);

    // Solve each state's weak system with shared, deterministic STLSQ settings.
    let solve_config = StlsqConfig {
        threshold: config.threshold,
        ridge: config.ridge,
        max_iterations: config.max_iterations,
    };
    let mut coefficients = Vec::with_capacity(states.len());
    let mut residuals = Vec::with_capacity(states.len());
    for target in &system.targets {
        let fit = stlsq(&system.library, target, &solve_config)?;
        residuals.push(fit.residual);
        coefficients.push(fit.coefficients);
    }

    let laws = build_laws(&state_names, &term_names, &coefficients);
    let max_residual = residuals.iter().cloned().fold(0.0_f64, f64::max);
    let diagnostics = WeakDiagnostics {
        test_functions: tests.len(),
        library_terms: term_names.len(),
        condition: conditioning(&system.library, config.ridge),
        residuals,
        max_residual,
    };

    Ok(WeakResult { laws, coefficients, term_names, state_names, diagnostics })
}

fn build_laws(
    state_names: &[String],
    term_names: &[String],
    coefficients: &[Vec<f64>],
) -> Vec<WeakLaw> {
    state_names
        .iter()
        .zip(coefficients)
        .map(|(state, row)| {
            let terms = row
                .iter()
                .zip(term_names)
                .filter(|(coefficient, _)| **coefficient != 0.0)
                .map(|(coefficient, name)| WeakTerm {
                    name: name.clone(),
                    coefficient: *coefficient,
                })
                .collect();
            WeakLaw { state: state.clone(), terms }
        })
        .collect()
}
