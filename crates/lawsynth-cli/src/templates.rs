//! `lawsynth new` and `lawsynth templates` — a catalog of canonical dynamical
//! systems users can start from.
//!
//! Each template constructs a real World IR, writes a `.lsworld` bundle, and can
//! generate a deterministic synthetic observation CSV (by simulating the true
//! system with fixed initial conditions and step) that a user can immediately
//! `discover`.

use std::fmt::Write as _;
use std::fs;

use lawsynth_bundle::write_world;
use lawsynth_core::Identifier;
use lawsynth_expr::{Expr, UnaryOperator};
use lawsynth_sim::{SimulationConfig, SimulationRequest, simulate};
use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

/// The true World IR paired with its default initial state.
type BuiltSystem = (World, Vec<(&'static str, f64)>);

/// A canonical system a user can instantiate with `lawsynth new`.
struct Template {
    name: &'static str,
    description: &'static str,
    /// Fixed integration step used when generating synthetic observations.
    step: f64,
    /// Default number of observation rows to emit with `--data`.
    default_samples: usize,
    /// Builds the true World IR and its default initial state.
    build: fn() -> BuiltSystem,
}

fn id(value: &str) -> Identifier {
    Identifier::new(value).expect("template identifiers are valid by construction")
}

fn state(name: &str) -> Variable {
    Variable::new(id(name), VariableRole::State)
}

fn param(name: &str, value: f64) -> Parameter {
    Parameter::new(id(name), value)
}

fn sym(name: &str) -> Expr {
    Expr::symbol(id(name))
}

fn law(target: &str, expression: Expr) -> ContinuousLaw {
    ContinuousLaw::new(id(target), expression)
}

/// `x^2` as an explicit product (keeps the discovered/interchange form simple).
fn square(name: &str) -> Expr {
    Expr::product(sym(name), sym(name))
}

fn build_lorenz() -> BuiltSystem {
    // dx/dt = sigma*(y - x)
    // dy/dt = x*(rho - z) - y
    // dz/dt = x*y - beta*z
    let world = World::new(
        [state("x"), state("y"), state("z")],
        [param("sigma", 10.0), param("rho", 28.0), param("beta", 8.0 / 3.0)],
        [
            law("x", Expr::product(sym("sigma"), Expr::difference(sym("y"), sym("x")))),
            law(
                "y",
                Expr::difference(
                    Expr::product(sym("x"), Expr::difference(sym("rho"), sym("z"))),
                    sym("y"),
                ),
            ),
            law(
                "z",
                Expr::difference(
                    Expr::product(sym("x"), sym("y")),
                    Expr::product(sym("beta"), sym("z")),
                ),
            ),
        ],
    )
    .expect("lorenz world is well-formed");
    (world, vec![("x", 1.0), ("y", 1.0), ("z", 1.0)])
}

fn build_lotka_volterra() -> BuiltSystem {
    // dx/dt = alpha*x - beta*x*y      (prey)
    // dy/dt = delta*x*y - gamma*y     (predator)
    let world = World::new(
        [state("x"), state("y")],
        [param("alpha", 1.1), param("beta", 0.4), param("delta", 0.1), param("gamma", 0.4)],
        [
            law(
                "x",
                Expr::difference(
                    Expr::product(sym("alpha"), sym("x")),
                    Expr::product(sym("beta"), Expr::product(sym("x"), sym("y"))),
                ),
            ),
            law(
                "y",
                Expr::difference(
                    Expr::product(sym("delta"), Expr::product(sym("x"), sym("y"))),
                    Expr::product(sym("gamma"), sym("y")),
                ),
            ),
        ],
    )
    .expect("lotka-volterra world is well-formed");
    (world, vec![("x", 10.0), ("y", 5.0)])
}

fn build_pendulum() -> BuiltSystem {
    // Damped pendulum:
    //   d(theta)/dt = omega
    //   d(omega)/dt = -damping*omega - freq*sin(theta)
    let world = World::new(
        [state("theta"), state("omega")],
        [param("damping", 0.25), param("freq", 5.0)],
        [
            law("theta", sym("omega")),
            law(
                "omega",
                Expr::difference(
                    Expr::product(Expr::unary(UnaryOperator::Negate, sym("damping")), sym("omega")),
                    Expr::product(sym("freq"), Expr::unary(UnaryOperator::Sin, sym("theta"))),
                ),
            ),
        ],
    )
    .expect("pendulum world is well-formed");
    // Release the pendulum from horizontal (theta = pi/2) at rest.
    (world, vec![("theta", std::f64::consts::FRAC_PI_2), ("omega", 0.0)])
}

fn build_van_der_pol() -> BuiltSystem {
    // dx/dt = y
    // dy/dt = mu*(1 - x^2)*y - x
    let world = World::new(
        [state("x"), state("y")],
        [param("mu", 1.0)],
        [
            law("x", sym("y")),
            law(
                "y",
                Expr::difference(
                    Expr::product(
                        sym("mu"),
                        Expr::product(Expr::difference(Expr::constant(1.0), square("x")), sym("y")),
                    ),
                    sym("x"),
                ),
            ),
        ],
    )
    .expect("van-der-pol world is well-formed");
    (world, vec![("x", 2.0), ("y", 0.0)])
}

fn build_sir() -> BuiltSystem {
    // Normalized SIR (S + I + R = 1):
    //   dS/dt = -beta*S*I
    //   dI/dt =  beta*S*I - gamma*I
    //   dR/dt =  gamma*I
    let world = World::new(
        [state("S"), state("I"), state("R")],
        [param("beta", 0.6), param("gamma", 0.1)],
        [
            law(
                "S",
                Expr::product(
                    Expr::unary(UnaryOperator::Negate, sym("beta")),
                    Expr::product(sym("S"), sym("I")),
                ),
            ),
            law(
                "I",
                Expr::difference(
                    Expr::product(sym("beta"), Expr::product(sym("S"), sym("I"))),
                    Expr::product(sym("gamma"), sym("I")),
                ),
            ),
            law("R", Expr::product(sym("gamma"), sym("I"))),
        ],
    )
    .expect("sir world is well-formed");
    (world, vec![("S", 0.99), ("I", 0.01), ("R", 0.0)])
}

const TEMPLATES: &[Template] = &[
    Template {
        name: "lorenz",
        description: "Lorenz attractor - 3D chaotic convection (x,y,z; sigma,rho,beta)",
        step: 0.01,
        default_samples: 2000,
        build: build_lorenz,
    },
    Template {
        name: "lotka-volterra",
        description: "Lotka-Volterra predator-prey oscillator (x,y; alpha,beta,delta,gamma)",
        step: 0.05,
        default_samples: 400,
        build: build_lotka_volterra,
    },
    Template {
        name: "pendulum",
        description: "Damped nonlinear pendulum (theta,omega; damping,freq) - needs --trigonometric",
        step: 0.02,
        default_samples: 500,
        build: build_pendulum,
    },
    Template {
        name: "van-der-pol",
        description: "Van der Pol relaxation oscillator (x,y; mu) - needs --degree 3",
        step: 0.05,
        default_samples: 400,
        build: build_van_der_pol,
    },
    Template {
        name: "sir",
        description: "SIR epidemic compartments, normalized (S,I,R; beta,gamma)",
        step: 0.5,
        default_samples: 160,
        build: build_sir,
    },
];

/// Help text for `lawsynth new`.
pub fn new_help() -> String {
    let mut out = String::from(
        "lawsynth new <template> [--output WORLD.lsworld] [--data OBS.csv] [--samples N]\n\n\
Instantiates a canonical system as a real .lsworld bundle. With --data it also \
writes a deterministic synthetic observation CSV (time + state columns) produced \
by simulating the true system, ready to `discover`.\n\n\
Templates:\n",
    );
    for template in TEMPLATES {
        let _ = writeln!(out, "  {:<15} {}", template.name, template.description);
    }
    out
}

/// Help text for `lawsynth templates`.
pub fn templates_help() -> String {
    "lawsynth templates\n\n\
Lists the canonical systems available to `lawsynth new`."
        .to_owned()
}

/// Runs the `templates` command: lists the catalog.
pub fn run_templates(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(templates_help());
    }
    if !arguments.is_empty() {
        return Err(templates_help());
    }
    let mut out = String::from("Available templates:\n");
    for template in TEMPLATES {
        let _ = writeln!(out, "  {:<15} {}", template.name, template.description);
    }
    Ok(out)
}

struct NewArgs {
    template: String,
    output: Option<String>,
    data: Option<String>,
    samples: Option<usize>,
}

/// Runs the `new` command: instantiates a template.
pub fn run_new(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(new_help());
    }
    let args = parse_new(arguments)?;
    let Some(template) = TEMPLATES.iter().find(|candidate| candidate.name == args.template) else {
        return Err(format!("unknown template '{}'\n\n{}", args.template, templates_help()));
    };

    let (world, initial) = (template.build)();
    let output = args.output.unwrap_or_else(|| format!("{}.lsworld", template.name));
    write_world(&output, &world).map_err(|error| error.to_string())?;

    let mut summary = format!(
        "created world: {output} ({} state(s), {} parameter(s), template '{}')\n",
        world.state_ids().count(),
        world.parameters().len(),
        template.name
    );

    if let Some(data_path) = &args.data {
        let samples = args.samples.unwrap_or(template.default_samples);
        let csv = generate_observations(&world, &initial, template.step, samples)?;
        fs::write(data_path, &csv)
            .map_err(|error| format!("failed to write {data_path}: {error}"))?;
        let _ = writeln!(
            summary,
            "generated observations: {data_path} ({samples} rows, step {}, columns time,{})",
            template.step,
            world.state_ids().map(Identifier::as_str).collect::<Vec<_>>().join(",")
        );
    }
    Ok(summary)
}

fn parse_new(arguments: &[String]) -> Result<NewArgs, String> {
    let Some(template) = arguments.first() else {
        return Err(new_help());
    };
    if template.starts_with('-') {
        return Err(new_help());
    }
    let mut args = NewArgs { template: template.clone(), output: None, data: None, samples: None };
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--output" => args.output = Some(value.clone()),
            "--data" => args.data = Some(value.clone()),
            "--samples" => {
                let count: usize =
                    value.parse().map_err(|_| format!("invalid sample count '{value}'"))?;
                if count < 2 {
                    return Err("--samples must be at least 2".to_owned());
                }
                args.samples = Some(count);
            }
            _ => return Err(new_help()),
        }
        index += 2;
    }
    Ok(args)
}

/// Simulates the world deterministically and renders a `discover`-ready CSV.
fn generate_observations(
    world: &World,
    initial: &[(&'static str, f64)],
    step: f64,
    samples: usize,
) -> Result<String, String> {
    let end = step * (samples - 1) as f64;
    let mut request = SimulationRequest::default();
    for (name, value) in initial {
        request = request.with_initial(id(name), *value);
    }
    let config = SimulationConfig::new(0.0, end, step).map_err(|error| error.to_string())?;
    let trajectory = simulate(world, config, &request).map_err(|error| error.to_string())?;

    let state_ids: Vec<&Identifier> = world.state_ids().collect();
    let mut csv = String::from("time");
    for state in &state_ids {
        let _ = write!(csv, ",{}", state.as_str());
    }
    csv.push('\n');
    // Emit exactly `samples` rows so the grid is regular for downstream tools.
    let rows = trajectory.samples().min(samples);
    for row in 0..rows {
        let _ = write!(csv, "{:.12e}", trajectory.time[row]);
        for state in &state_ids {
            let value = trajectory.values[*state][row];
            let _ = write!(csv, ",{value:.12e}");
        }
        csv.push('\n');
    }
    Ok(csv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_template_builds_a_valid_world() {
        for template in TEMPLATES {
            let (world, initial) = (template.build)();
            assert!(world.state_ids().count() >= 2, "template {} has states", template.name);
            assert_eq!(initial.len(), world.state_ids().count());
        }
    }

    #[test]
    fn generates_a_regular_observation_grid() {
        let (world, initial) = build_lotka_volterra();
        let csv = generate_observations(&world, &initial, 0.05, 10).unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(lines[0], "time,x,y");
        // header + 10 data rows
        assert_eq!(lines.len(), 11);
    }

    #[test]
    fn lists_all_templates() {
        let listing = run_templates(&[]).unwrap();
        for template in TEMPLATES {
            assert!(listing.contains(template.name));
        }
    }

    #[test]
    fn rejects_unknown_template() {
        let error = run_new(&["nope".to_owned()]).unwrap_err();
        assert!(error.contains("unknown template"));
    }
}
