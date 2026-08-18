//! Top-level orchestration: sweep → sample → branch → detect.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_stability::StabilityConfig;

use crate::branch::assemble_branches;
use crate::context::FieldContext;
use crate::detect::{deduplicate, detect_crossings, detect_folds};
use crate::error::BifurcationError;
use crate::report::{ContinuationReport, ParameterSample};
use crate::sweep::Sweep;

/// Sweeps a scalar `parameter` across a field, tracking fixed-point branches and
/// detecting bifurcations.
///
/// The pipeline is deterministic and offline:
///
/// 1. **Sample.** On the fixed grid of `sweep`, substitute each parameter value
///    into the field and locate/classify fixed points with
///    [`lawsynth_stability::analyze_stability`].
/// 2. **Branch.** Stitch fixed points across consecutive samples into branches by
///    nearest-coordinate matching.
/// 3. **Detect.** Flag where a branch's dominant eigenvalue crosses the imaginary
///    axis (Hopf if complex, a real zero-eigenvalue fold otherwise) and where a
///    branch is born/destroyed with a near-zero eigenvalue (a collision fold).
///    Critical values are localized by deterministic bisection and merged.
///
/// Identical inputs yield a bit-identical [`ContinuationReport`].
///
/// # Errors
///
/// Returns [`BifurcationError::EmptyStateSpace`] if `states` is empty,
/// [`BifurcationError::ParameterIsState`] if the parameter is also a state,
/// [`BifurcationError::InvalidSweep`] if the sweep is ill-formed, and
/// [`BifurcationError::Stability`] if fixed-point analysis fails at some value.
pub fn continuation(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    parameter: &Identifier,
    sweep: &Sweep,
    stability: &StabilityConfig,
) -> Result<ContinuationReport, BifurcationError> {
    if states.is_empty() {
        return Err(BifurcationError::EmptyStateSpace);
    }
    if states.iter().any(|state| state == parameter) {
        return Err(BifurcationError::ParameterIsState(parameter.clone()));
    }
    sweep.validate()?;

    let context = FieldContext::new(fields, states, parameter, stability);
    let grid = sweep.grid();

    // 1. Sample the field on the parameter grid.
    let mut samples = Vec::with_capacity(grid.len());
    for &parameter_value in &grid {
        let report = context.at(parameter_value)?;
        samples.push(ParameterSample { parameter_value, report });
    }

    // 2. Assemble branches by nearest-coordinate matching.
    let (branches, spans) = assemble_branches(&samples, sweep.match_tolerance());

    // 3. Detect and localize bifurcations from both detectors, then merge.
    let counts: Vec<usize> =
        samples.iter().map(|sample| sample.report.fixed_points.len()).collect();
    let mut candidates = detect_crossings(&context, sweep, &branches)?;
    candidates.extend(detect_folds(&context, sweep, &counts, &grid, &branches, &spans)?);
    let bifurcations = deduplicate(candidates, sweep);

    Ok(ContinuationReport {
        states: states.to_vec(),
        parameter: parameter.clone(),
        samples,
        branches,
        bifurcations,
    })
}
