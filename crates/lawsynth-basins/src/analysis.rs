//! Orchestration: attractors → initial-condition grid → forward flow → labels.

use lawsynth_core::Identifier;
use lawsynth_expr::Expr;
use lawsynth_stability::{Classification, analyze_stability};

use crate::classify::classify_trajectory;
use crate::config::BasinConfig;
use crate::error::BasinError;
use crate::grid::initial_conditions;
use crate::integrate::Flow;
use crate::report::{Attractor, BasinReport, Label};

/// Maps the basins of attraction of an autonomous vector field.
///
/// `fields` pairs each state with its right-hand side `dx_i/dt = f_i(x)`;
/// `states` fixes the coordinate ordering. The procedure:
///
/// 1. locates the STABLE attractors by delegating to
///    [`lawsynth_stability::analyze_stability`] and keeping only the stable
///    nodes and stable spirals;
/// 2. lays a deterministic grid of initial conditions over the search box;
/// 3. integrates each initial condition forward with fixed-step RK4; and
/// 4. classifies each trajectory's fate — the attractor it reached, or the
///    honest `Escaped` / `Undetermined` outcomes.
///
/// The result is deterministic: identical inputs yield a bit-identical report.
///
/// # Errors
///
/// See [`BasinError`]. Attractor detection carries through `lawsynth-stability`
/// errors (notably a non-autonomous field yields
/// [`lawsynth_stability::StabilityError::UnknownSymbol`], wrapped in
/// [`BasinError::Stability`]). A field with no recognized stable attractor is not
/// an error: it produces an honest report with no attractors and every
/// trajectory escaped or undetermined.
pub fn map_basins(
    fields: &[(Identifier, Expr)],
    states: &[Identifier],
    config: &BasinConfig,
) -> Result<BasinReport, BasinError> {
    if states.is_empty() {
        return Err(BasinError::EmptyStateSpace);
    }
    config.validate(states.len())?;

    // Locate the fixed points and their linear-stability class. This also
    // validates the field structure and autonomy, so any structural fault or
    // unknown symbol surfaces here rather than mid-integration.
    let stability = analyze_stability(fields, states, config.stability_config())?;

    // Attractors are exactly the stable fixed points, in stability's order.
    let attractors: Vec<Attractor> = stability
        .fixed_points
        .iter()
        .filter(|point| is_attractor(point.classification))
        .map(|point| Attractor {
            coordinates: point.coordinates.clone(),
            classification: point.classification,
        })
        .collect();
    let attractor_coordinates: Vec<Vec<f64>> =
        attractors.iter().map(|attractor| attractor.coordinates.clone()).collect();

    // The field expressions aligned to `states`. Stability already proved the
    // field is well-formed, so this ordering cannot fail in practice; a missing
    // field is surfaced as a stability error for a single, honest error channel.
    let ordered_fields = order_fields(fields, states)?;
    let flow = Flow::new(states, ordered_fields);

    let grid = initial_conditions(config.search_box(), config.grid_resolution());
    let mut grid_labels = Vec::with_capacity(grid.len());
    let mut counts = vec![0usize; attractors.len()];
    let mut escaped = 0usize;
    let mut undetermined = 0usize;

    for initial in &grid {
        let label = classify_trajectory(&flow, initial, &attractor_coordinates, config);
        match label {
            Label::Attractor(index) => counts[index] += 1,
            Label::Escaped => escaped += 1,
            Label::Undetermined => undetermined += 1,
        }
        grid_labels.push(label);
    }

    let settled: usize = counts.iter().sum();
    let fractions = counts
        .iter()
        .map(|&count| if settled == 0 { 0.0 } else { count as f64 / settled as f64 })
        .collect();

    Ok(BasinReport {
        states: states.to_vec(),
        attractors,
        grid_labels,
        fractions,
        escaped,
        undetermined,
        resolution: config.grid_resolution(),
        search_box: config.search_box().to_vec(),
    })
}

/// Whether a linear-stability class is a recognized attractor. Only stable nodes
/// and stable spirals attract; saddles, unstable points, and non-hyperbolic
/// (`Center`/`Marginal`) points do not.
fn is_attractor(classification: Classification) -> bool {
    matches!(classification, Classification::StableNode | Classification::StableSpiral)
}

/// The field expressions in `states` order.
fn order_fields<'a>(
    fields: &'a [(Identifier, Expr)],
    states: &[Identifier],
) -> Result<Vec<&'a Expr>, BasinError> {
    states
        .iter()
        .map(|state| {
            fields
                .iter()
                .find(|(target, _)| target == state)
                .map(|(_, expression)| expression)
                .ok_or_else(|| {
                    BasinError::Stability(lawsynth_stability::StabilityError::Jacobian(
                        lawsynth_jacobian::JacobianError::MissingField(state.clone()),
                    ))
                })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attractor_predicate_accepts_only_stable_classes() {
        assert!(is_attractor(Classification::StableNode));
        assert!(is_attractor(Classification::StableSpiral));
        assert!(!is_attractor(Classification::Saddle));
        assert!(!is_attractor(Classification::UnstableNode));
        assert!(!is_attractor(Classification::UnstableSpiral));
        assert!(!is_attractor(Classification::Center));
        assert!(!is_attractor(Classification::Marginal));
    }
}
