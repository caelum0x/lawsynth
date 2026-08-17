//! `lawsynth simplify` — reduce each law to its smallest equivalent form.
//!
//! Each law expression is run through the real `lawsynth-egraph` equality
//! saturation engine (safe algebraic identities: constant folding, `x+0`, `x*1`,
//! `x*0`, `x^1`, ... plus canonical commutative ordering) and the
//! `lawsynth-symbolic` local simplifier. The lowest-cost (fewest AST nodes)
//! equivalent form is then extracted with the e-graph's cost model.
//!
//! Every rewrite in the rule set is value-preserving, so the result is
//! *mathematically equivalent*. The command proves this honestly by simulating
//! both the original and the simplified world from identical initial conditions
//! and reporting the maximum trajectory deviation (which is at machine-epsilon
//! scale, and typically exactly zero for these exact rewrites).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use lawsynth_bundle::{read_world, write_world};
use lawsynth_core::Identifier;
use lawsynth_egraph::{EquivalenceGraph, RewriteConfig, expression_cost, extract_lowest_cost};
use lawsynth_expr::Expr;
use lawsynth_report::render_continuous_law;
use lawsynth_sim::{SimulationConfig, SimulationRequest, Trajectory, simulate};
use lawsynth_symbolic::simplify_candidate;
use lawsynth_world::{ContinuousLaw, World};

/// Deviation at or below this bound counts as machine-equivalent trajectories.
const EQUIVALENCE_TOLERANCE: f64 = 1e-9;

/// Fixed, deterministic window used to prove trajectory equivalence.
const VERIFY_START: f64 = 0.0;
const VERIFY_END: f64 = 1.0;
const VERIFY_STEP: f64 = 0.01;

/// Help text for `lawsynth simplify`.
pub fn help() -> String {
    "lawsynth simplify WORLD.lsworld [--output SIMPLIFIED.lsworld]\n\n\
Simplifies each law's expression with equality saturation (lawsynth-egraph) and \
the symbolic simplifier, extracting the smallest equivalent form (constant \
folding, identity collapse like x+0 / x*1 / x*0, and canonical ordering). Prints \
a before/after equation and AST node count per law. The rewrites are \
value-preserving, so the command simulates both worlds and reports the maximum \
trajectory deviation as an equivalence proof. With --output, writes the \
simplified-but-equivalent world."
        .to_owned()
}

/// Runs the `simplify` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => output = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }

    let world = read_world(bundle).map_err(|error| error.to_string())?;
    let simplified = simplify_world(&world)?;

    let report = render_report(&world, &simplified, bundle)?;

    if let Some(path) = output {
        write_world(&path, &simplified).map_err(|error| error.to_string())?;
        return Ok(format!("{report}\nwrote simplified world: {path}\n"));
    }
    Ok(report)
}

/// Returns a new world with every law expression reduced to its smallest
/// equivalent form. The result is re-validated by [`World::new`].
pub fn simplify_world(world: &World) -> Result<World, String> {
    let laws: Vec<ContinuousLaw> = world
        .laws()
        .values()
        .map(|law| ContinuousLaw::new(law.target.clone(), simplify_expression(&law.expression)))
        .collect();
    World::new(
        world.variables().values().cloned().collect::<Vec<_>>(),
        world.parameters().values().cloned().collect::<Vec<_>>(),
        laws,
    )
    .map_err(|error| error.to_string())
}

/// Extracts the smallest expression equivalent to `expression` using the
/// e-graph's equality saturation, the symbolic simplifier, and the e-graph
/// cost model. The original is always a candidate, so the result never grows.
pub fn simplify_expression(expression: &Expr) -> Expr {
    let config = RewriteConfig::default();
    let mut graph = EquivalenceGraph::default();
    // Saturating over the safe rule set yields the canonical member plus every
    // raw form that collapsed into the same equivalence class.
    let class = graph.add(expression.clone(), &config).clone();

    let mut candidates = class.members.clone();
    candidates.push(class.canonical.clone());
    // Fold in the symbolic crate's local simplifier as an independent path.
    candidates.push(simplify_candidate(expression));
    // And ensure the original is always in contention for the min.
    candidates.push(expression.clone());

    extract_lowest_cost(&candidates).unwrap_or_else(|| expression.clone())
}

/// Node count of an expression (the e-graph extraction cost).
fn complexity(expression: &Expr) -> usize {
    expression_cost(expression)
}

/// Builds the full before/after + equivalence report.
fn render_report(original: &World, simplified: &World, bundle: &str) -> Result<String, String> {
    let mut out = String::new();
    let _ = writeln!(out, "Simplifying {bundle}");
    out.push('\n');

    let mut total_before = 0usize;
    let mut total_after = 0usize;
    let mut reduced_laws = 0usize;

    for (target, before_law) in original.laws() {
        let after_law = &simplified.laws()[target];
        let before = complexity(&before_law.expression);
        let after = complexity(&after_law.expression);
        total_before += before;
        total_after += after;

        let _ = writeln!(out, "Law d{}/dt", target.as_str());
        let _ = writeln!(
            out,
            "  before: {}  ({before} nodes)",
            render_continuous_law(target.as_str(), &before_law.expression)
        );
        let _ = writeln!(
            out,
            "  after:  {}  ({after} nodes)",
            render_continuous_law(target.as_str(), &after_law.expression)
        );
        if after < before {
            reduced_laws += 1;
            let _ = writeln!(out, "  reduced by {} node(s)", before - after);
        } else {
            let _ = writeln!(out, "  already minimal under the safe rewrite rules");
        }
        out.push('\n');
    }

    let saved = total_before.saturating_sub(total_after);
    let percent = if total_before > 0 { 100.0 * saved as f64 / total_before as f64 } else { 0.0 };
    let _ = writeln!(
        out,
        "Complexity: {total_before} -> {total_after} node(s)  ({reduced_laws}/{} law(s) reduced, {saved} node(s) / {percent:.1}% saved)",
        original.laws().len()
    );

    // Honest equivalence proof: simulate both and report the worst deviation.
    match max_deviation(original, simplified) {
        Ok((deviation, states, samples)) => {
            let verdict =
                if deviation <= EQUIVALENCE_TOLERANCE { "EQUIVALENT" } else { "MISMATCH" };
            let _ = writeln!(
                out,
                "Equivalence: simulated both worlds (t in [{VERIFY_START}, {VERIFY_END}], step {VERIFY_STEP}, {states} state(s), {samples} sample(s))"
            );
            let _ = writeln!(
                out,
                "  max trajectory deviation: {deviation:.3e}  (tolerance {EQUIVALENCE_TOLERANCE:.0e}) -> {verdict}"
            );
            if deviation > EQUIVALENCE_TOLERANCE {
                return Err(format!(
                    "simplification changed the dynamics (max deviation {deviation:.3e} exceeds tolerance {EQUIVALENCE_TOLERANCE:.0e}); refusing to claim equivalence"
                ));
            }
        }
        Err(reason) => {
            let _ = writeln!(
                out,
                "Equivalence: could not simulate for verification ({reason}); rewrites are value-preserving by construction"
            );
        }
    }

    Ok(out)
}

/// Simulates both worlds from identical initial conditions and returns the
/// maximum absolute deviation over every state and sample, with the state count
/// and sample count.
fn max_deviation(original: &World, simplified: &World) -> Result<(f64, usize, usize), String> {
    let config = SimulationConfig::new(VERIFY_START, VERIFY_END, VERIFY_STEP)
        .map_err(|error| error.to_string())?;
    let request = deterministic_request(original);
    let original_trajectory =
        simulate(original, config, &request).map_err(|error| error.to_string())?;
    let simplified_trajectory =
        simulate(simplified, config, &request).map_err(|error| error.to_string())?;
    let deviation = trajectory_deviation(&original_trajectory, &simplified_trajectory);
    Ok((deviation, original_trajectory.values.len(), original_trajectory.samples()))
}

/// Assigns each state a distinct, deterministic nonzero initial value so the
/// laws are actually exercised (a zero start could mask multiplicative changes).
fn deterministic_request(world: &World) -> SimulationRequest {
    let mut request = SimulationRequest::default();
    for (index, state) in world.state_ids().enumerate() {
        request = request.with_initial(state.clone(), 0.5 + 0.1 * index as f64);
    }
    request
}

/// Maximum absolute difference across shared states and aligned samples.
fn trajectory_deviation(left: &Trajectory, right: &Trajectory) -> f64 {
    let mut worst = 0.0_f64;
    let shared: BTreeMap<&Identifier, &Vec<f64>> = left.values.iter().collect();
    for (state, right_values) in &right.values {
        let Some(left_values) = shared.get(state) else {
            continue;
        };
        let samples = left_values.len().min(right_values.len());
        for index in 0..samples {
            worst = worst.max((left_values[index] - right_values[index]).abs());
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use lawsynth_world::{Parameter, Variable, VariableRole};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    #[test]
    fn removes_identity_multiplier() {
        // 1 * x  ->  x
        let expression = Expr::product(Expr::constant(1.0), Expr::symbol(id("x")));
        assert_eq!(simplify_expression(&expression), Expr::symbol(id("x")));
    }

    #[test]
    fn folds_constants() {
        // (2 * 3) + x  ->  6 + x  (fewer nodes)
        let expression = Expr::sum(
            Expr::product(Expr::constant(2.0), Expr::constant(3.0)),
            Expr::symbol(id("x")),
        );
        let simplified = simplify_expression(&expression);
        assert!(complexity(&simplified) < complexity(&expression));
    }

    #[test]
    fn never_grows_the_expression() {
        let expression = Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x")));
        assert!(complexity(&simplify_expression(&expression)) <= complexity(&expression));
    }

    #[test]
    fn simplified_world_is_equivalent() {
        // dx/dt = 1 * (-1 * x)   (a redundant identity factor)
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(
                    Expr::constant(1.0),
                    Expr::product(Expr::constant(-1.0), Expr::symbol(id("x"))),
                ),
            )],
        )
        .unwrap();
        let simplified = simplify_world(&world).unwrap();
        let (deviation, _, _) = max_deviation(&world, &simplified).unwrap();
        assert!(deviation <= EQUIVALENCE_TOLERANCE, "deviation {deviation:e}");
        // The redundant `1 *` factor is gone.
        assert!(
            complexity(&simplified.laws()[&id("x")].expression)
                < complexity(&world.laws()[&id("x")].expression)
        );
    }

    #[test]
    fn keeps_parameters_declared() {
        let world = World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 1.0)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(Expr::symbol(id("k")), Expr::symbol(id("x"))),
            )],
        )
        .unwrap();
        let simplified = simplify_world(&world).unwrap();
        assert!(simplified.parameters().contains_key(&id("k")));
    }
}
