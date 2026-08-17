//! `lawsynth export` — "use your model anywhere".
//!
//! Turns a discovered `.lsworld` into a standalone, dependency-free artifact so
//! the world runs in any environment: a runnable Python module, a compilable C
//! source (`lawsynth_derivatives` + RK4 + `main`), an ONNX-style computation
//! graph JSON, an Octave/MATLAB `.m` file, a LaTeX `align*` law system, or a
//! documented JSON description of the world. The per-expression emitters live
//! in `lawsynth-report` so every surface renders equations the same way.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::fs;

use lawsynth_bundle::read_world;
use lawsynth_core::Identifier;
use lawsynth_report::{
    build_computation_graph, format_number, python_number, render_c_expression,
    render_computation_graph_json, render_expression, render_latex_expression, render_latex_law,
    render_matlab_expression, render_python_expression,
};
use lawsynth_world::{VariableRole, World};

/// Help text for `lawsynth export`.
pub fn help() -> String {
    "lawsynth export WORLD.lsworld --format <python|c|onnx|matlab|latex|json> [--output FILE]\n\n\
Generates a standalone artifact from a discovered world:\n  \
python  a dependency-free module with derivatives(t, state, params) and an RK4 simulate(...)\n  \
c       a dependency-free C source: lawsynth_derivatives(t, state, dstate), RK4, and a main()\n  \
onnx    a LawSynth computation-graph JSON (ONNX-style op DAG; one output per state derivative)\n  \
matlab  an Octave/MATLAB .m file: function dstate = lawsynth_derivatives(t, state) + a demo\n  \
latex   the law system as an align* block of \\dot{x} = ... equations\n  \
json    a documented JSON description of variables, parameters, and laws\n\n\
Writes to --output when given, otherwise prints to stdout."
        .to_owned()
}

enum Format {
    Python,
    C,
    Onnx,
    Matlab,
    Latex,
    Json,
}

struct ExportArgs {
    bundle: String,
    format: Format,
    output: Option<String>,
}

/// Runs the `export` command.
pub fn run(arguments: &[String]) -> Result<String, String> {
    if matches!(arguments.first().map(String::as_str), Some("--help" | "-h")) {
        return Ok(help());
    }
    let args = parse(arguments)?;
    let world = read_world(&args.bundle).map_err(|error| error.to_string())?;
    let stem = std::path::Path::new(&args.bundle)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("world")
        .to_owned();

    let artifact = match args.format {
        Format::Python => emit_python(&world, &stem),
        Format::C => emit_c(&world, &stem),
        Format::Onnx => emit_onnx(&world, &stem),
        Format::Matlab => emit_matlab(&world, &stem),
        Format::Latex => emit_latex(&world, &stem),
        Format::Json => emit_json(&world, &stem),
    };

    match &args.output {
        Some(path) => {
            fs::write(path, &artifact)
                .map_err(|error| format!("failed to write {path}: {error}"))?;
            Ok(format!("wrote {} ({} bytes)\n", path, artifact.len()))
        }
        None => Ok(artifact),
    }
}

fn parse(arguments: &[String]) -> Result<ExportArgs, String> {
    let Some(bundle) = arguments.first() else {
        return Err(help());
    };
    if bundle.starts_with('-') {
        return Err(help());
    }
    let mut format = None;
    let mut output = None;
    let mut index = 1;
    while index < arguments.len() {
        let option = arguments[index].as_str();
        let value =
            arguments.get(index + 1).ok_or_else(|| format!("missing value for {option}"))?;
        match option {
            "--format" => {
                format = Some(match value.as_str() {
                    "python" | "py" => Format::Python,
                    "c" => Format::C,
                    "onnx" | "graph" => Format::Onnx,
                    "matlab" | "octave" | "m" => Format::Matlab,
                    "latex" | "tex" => Format::Latex,
                    "json" => Format::Json,
                    other => {
                        return Err(format!(
                            "unknown format '{other}'; use python|c|onnx|matlab|latex|json"
                        ));
                    }
                });
            }
            "--output" => output = Some(value.clone()),
            _ => return Err(help()),
        }
        index += 2;
    }
    Ok(ExportArgs {
        bundle: bundle.clone(),
        format: format.ok_or("missing required --format <python|c|onnx|matlab|latex|json>")?,
        output,
    })
}

/// Classifies every symbol a world's laws can reference.
///
/// Discovered worlds inline coefficients as constants and carry no parameters,
/// while template worlds carry named parameters; both are handled here.
fn state_ids(world: &World) -> BTreeSet<Identifier> {
    world.state_ids().cloned().collect()
}

// --- Python ---------------------------------------------------------------

pub(crate) fn emit_python(world: &World, name: &str) -> String {
    let states = state_ids(world);
    let mut out = String::new();

    let _ = writeln!(out, "\"\"\"Auto-generated by LawSynth from '{name}'.");
    out.push('\n');
    let _ = writeln!(out, "Dependency-free reproduction of the world's continuous-time dynamics.");
    let _ = writeln!(
        out,
        "`derivatives(t, state, params)` returns d(state)/dt; `simulate(...)` integrates"
    );
    let _ = writeln!(out, "the system with classical fourth-order Runge-Kutta (RK4).");
    let _ = writeln!(out, "\"\"\"");
    let _ = writeln!(out, "import math");
    out.push('\n');

    // Parameters carry their inline values. Non-state, non-parameter variables
    // (inputs) default to 0.0 so the module always runs.
    let _ = writeln!(out, "PARAMS = {{");
    for (id, parameter) in world.parameters() {
        let _ = writeln!(out, "    {:?}: {},", id.as_str(), python_number(parameter.value));
    }
    for variable in world.variables().values() {
        if variable.role != VariableRole::State && !world.parameters().contains_key(&variable.id) {
            let _ = writeln!(
                out,
                "    {:?}: 0.0,  # non-state input, edit as needed",
                variable.id.as_str()
            );
        }
    }
    let _ = writeln!(out, "}}");
    out.push('\n');

    let state_list = world.state_ids().map(|id| format!("{:?}", id.as_str())).collect::<Vec<_>>();
    let _ = writeln!(out, "STATE_VARS = [{}]", state_list.join(", "));
    out.push('\n');

    // Resolve a symbol onto its Python binding.
    let resolve = |id: &Identifier| -> String {
        if states.contains(id) {
            format!("state[{:?}]", id.as_str())
        } else {
            format!("params[{:?}]", id.as_str())
        }
    };

    let _ = writeln!(out, "def derivatives(t, state, params):");
    let _ = writeln!(out, "    \"\"\"Return d(state)/dt as a dict keyed by state name.\"\"\"");
    let _ = writeln!(out, "    return {{");
    for (target, law) in world.laws() {
        let expression = render_python_expression(&law.expression, &resolve);
        let _ = writeln!(out, "        {:?}: {},", target.as_str(), expression);
    }
    let _ = writeln!(out, "    }}");
    out.push('\n');

    // RK4 integrator.
    out.push_str(
        "def simulate(initial, t0, t1, dt, params=None):\n\
\x20   \"\"\"Integrate with RK4. Returns (times, trajectory) where\n\
\x20   trajectory[name] is the list of samples for each state variable.\"\"\"\n\
\x20   if params is None:\n\
\x20       params = dict(PARAMS)\n\
\x20   state = {name: float(initial[name]) for name in STATE_VARS}\n\
\x20   times = [t0]\n\
\x20   traj = {name: [state[name]] for name in STATE_VARS}\n\
\x20   steps = int(round((t1 - t0) / dt))\n\
\x20   t = t0\n\
\x20   for _ in range(steps):\n\
\x20       k1 = derivatives(t, state, params)\n\
\x20       s2 = {k: state[k] + 0.5 * dt * k1[k] for k in STATE_VARS}\n\
\x20       k2 = derivatives(t + 0.5 * dt, s2, params)\n\
\x20       s3 = {k: state[k] + 0.5 * dt * k2[k] for k in STATE_VARS}\n\
\x20       k3 = derivatives(t + 0.5 * dt, s3, params)\n\
\x20       s4 = {k: state[k] + dt * k3[k] for k in STATE_VARS}\n\
\x20       k4 = derivatives(t + dt, s4, params)\n\
\x20       state = {k: state[k] + dt * (k1[k] + 2.0 * k2[k] + 2.0 * k3[k] + k4[k]) / 6.0\n\
\x20                for k in STATE_VARS}\n\
\x20       t += dt\n\
\x20       times.append(t)\n\
\x20       for k in STATE_VARS:\n\
\x20           traj[k].append(state[k])\n\
\x20   return times, traj\n",
    );
    out.push('\n');

    // A runnable demo entry point.
    out.push_str(
        "if __name__ == \"__main__\":\n\
\x20   initial = {name: 1.0 for name in STATE_VARS}\n\
\x20   times, traj = simulate(initial, 0.0, 10.0, 0.01)\n\
\x20   print(\"time\", *STATE_VARS)\n\
\x20   print(times[-1], *[traj[name][-1] for name in STATE_VARS])\n",
    );

    out
}

// --- C --------------------------------------------------------------------

pub(crate) fn emit_c(world: &World, name: &str) -> String {
    // Ordered state layout: state[i] and dstate[i] follow this order.
    let states: Vec<&Identifier> = world.state_ids().collect();
    let mut out = String::new();

    let _ = writeln!(out, "/* Auto-generated by LawSynth from '{name}'.");
    let _ = writeln!(out, " *");
    let _ = writeln!(out, " * Standalone, dependency-free reproduction of the world's");
    let _ = writeln!(out, " * continuous-time dynamics dx/dt = f(x). Parameters are inlined as");
    let _ = writeln!(
        out,
        " * #define constants. `lawsynth_derivatives` evaluates f; `lawsynth_simulate`"
    );
    let _ = writeln!(out, " * integrates with classical fourth-order Runge-Kutta (RK4); `main`");
    let _ = writeln!(out, " * prints a short trajectory. Compile: cc -O2 {name}.c -lm -o {name}");
    let _ = writeln!(out, " */");
    let _ = writeln!(out, "#include <stdio.h>");
    let _ = writeln!(out, "#include <math.h>");
    out.push('\n');

    // Parameters and non-state inputs, inlined as macros.
    let _ = writeln!(out, "/* Parameters (inlined). */");
    for (id, parameter) in world.parameters() {
        let _ = writeln!(out, "#define P_{} ({})", id.as_str(), python_number(parameter.value));
    }
    for variable in world.variables().values() {
        if variable.role != VariableRole::State && !world.parameters().contains_key(&variable.id) {
            let _ = writeln!(
                out,
                "#define IN_{} (0.0) /* non-state input, edit as needed */",
                variable.id.as_str()
            );
        }
    }
    out.push('\n');

    let dim = states.len();
    let _ = writeln!(out, "enum {{ LAWSYNTH_STATE_DIM = {dim} }};");
    let names = states.iter().map(|id| format!("{:?}", id.as_str())).collect::<Vec<_>>();
    let _ = writeln!(
        out,
        "static const char *const LAWSYNTH_STATE_NAMES[LAWSYNTH_STATE_DIM] = {{ {} }};",
        names.join(", ")
    );
    out.push('\n');

    // Resolve a symbol onto its C binding.
    let resolve = |id: &Identifier| -> String {
        if let Some(index) = states.iter().position(|state| *state == id) {
            format!("state[{index}]")
        } else if world.parameters().contains_key(id) {
            format!("P_{}", id.as_str())
        } else {
            format!("IN_{}", id.as_str())
        }
    };

    // Derivative function.
    let _ = writeln!(
        out,
        "void lawsynth_derivatives(double t, const double *state, double *dstate) {{"
    );
    let _ = writeln!(out, "    (void)t;");
    for (index, state) in states.iter().enumerate() {
        match world.laws().get(*state) {
            Some(law) => {
                let expression = render_c_expression(&law.expression, &resolve);
                let _ = writeln!(out, "    dstate[{index}] = {expression};");
            }
            None => {
                let _ = writeln!(out, "    dstate[{index}] = 0.0;");
            }
        }
    }
    let _ = writeln!(out, "}}");
    out.push('\n');

    // One RK4 step, in place, and a full-span integrator.
    out.push_str(
        "void lawsynth_rk4_step(double t, double dt, double *state) {\n\
\x20   double k1[LAWSYNTH_STATE_DIM], k2[LAWSYNTH_STATE_DIM];\n\
\x20   double k3[LAWSYNTH_STATE_DIM], k4[LAWSYNTH_STATE_DIM];\n\
\x20   double tmp[LAWSYNTH_STATE_DIM];\n\
\x20   int i;\n\
\x20   lawsynth_derivatives(t, state, k1);\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) tmp[i] = state[i] + 0.5 * dt * k1[i];\n\
\x20   lawsynth_derivatives(t + 0.5 * dt, tmp, k2);\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) tmp[i] = state[i] + 0.5 * dt * k2[i];\n\
\x20   lawsynth_derivatives(t + 0.5 * dt, tmp, k3);\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) tmp[i] = state[i] + dt * k3[i];\n\
\x20   lawsynth_derivatives(t + dt, tmp, k4);\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i)\n\
\x20       state[i] += dt * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]) / 6.0;\n\
}\n\n\
void lawsynth_simulate(double *state, double t0, double t1, double dt) {\n\
\x20   int steps = (int)((t1 - t0) / dt + 0.5);\n\
\x20   int s;\n\
\x20   double t = t0;\n\
\x20   for (s = 0; s < steps; ++s) {\n\
\x20       lawsynth_rk4_step(t, dt, state);\n\
\x20       t += dt;\n\
\x20   }\n\
}\n\n",
    );

    // A runnable demo entry point that prints a short trajectory.
    out.push_str(
        "int main(void) {\n\
\x20   double state[LAWSYNTH_STATE_DIM];\n\
\x20   double t0 = 0.0, t1 = 1.0, dt = 0.01;\n\
\x20   int steps = (int)((t1 - t0) / dt + 0.5);\n\
\x20   int stride = steps / 10;\n\
\x20   int i, s;\n\
\x20   if (stride < 1) stride = 1;\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) state[i] = 1.0;\n\
\x20   double t = t0;\n\
\x20   printf(\"t\");\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) printf(\",%s\", LAWSYNTH_STATE_NAMES[i]);\n\
\x20   printf(\"\\n\");\n\
\x20   printf(\"%.17g\", t);\n\
\x20   for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) printf(\",%.17g\", state[i]);\n\
\x20   printf(\"\\n\");\n\
\x20   for (s = 0; s < steps; ++s) {\n\
\x20       lawsynth_rk4_step(t, dt, state);\n\
\x20       t += dt;\n\
\x20       if ((s + 1) % stride == 0) {\n\
\x20           printf(\"%.17g\", t);\n\
\x20           for (i = 0; i < LAWSYNTH_STATE_DIM; ++i) printf(\",%.17g\", state[i]);\n\
\x20           printf(\"\\n\");\n\
\x20       }\n\
\x20   }\n\
\x20   return 0;\n\
}\n",
    );

    out
}

// --- ONNX-style computation graph ----------------------------------------

fn emit_onnx(world: &World, name: &str) -> String {
    let graph = build_computation_graph(world);
    render_computation_graph_json(&graph, name)
}

// --- MATLAB / Octave ------------------------------------------------------

pub(crate) fn emit_matlab(world: &World, name: &str) -> String {
    let states: Vec<&Identifier> = world.state_ids().collect();
    let mut out = String::new();

    let _ = writeln!(out, "% Auto-generated by LawSynth from '{name}'.");
    let _ = writeln!(out, "%");
    let _ = writeln!(out, "% Octave/MATLAB reproduction of the world's continuous-time dynamics.");
    let _ = writeln!(out, "% `lawsynth_derivatives(t, state)` returns d(state)/dt as a column");
    let _ = writeln!(out, "% vector; `lawsynth_simulate(...)` integrates with classical RK4.");
    let _ = writeln!(out, "% Run this file (e.g. `octave {name}.m`) to print a short trajectory.");
    // The leading `1;` marks this as a script file (not a function file) so the
    // demo below runs and the local functions are still defined (Octave idiom;
    // MATLAB R2016b+ also allows local functions after script statements).
    let _ = writeln!(out, "1;");
    out.push('\n');

    // Runnable demo script.
    let dim = states.len();
    let names = states.iter().map(|id| format!("'{}'", id.as_str())).collect::<Vec<_>>();
    let _ = writeln!(out, "state_names = {{{}}};", names.join(", "));
    let _ = writeln!(out, "state0 = ones({dim}, 1);");
    let _ = writeln!(out, "[ts, traj] = lawsynth_simulate(state0, 0.0, 1.0, 0.01);");
    let _ = writeln!(out, "printf('t');");
    let _ = writeln!(out, "for i = 1:numel(state_names)");
    let _ = writeln!(out, "  printf(',%s', state_names{{i}});");
    let _ = writeln!(out, "end");
    let _ = writeln!(out, "printf('\\n');");
    let _ = writeln!(out, "stride = max(1, floor((numel(ts) - 1) / 10));");
    let _ = writeln!(out, "for k = 1:numel(ts)");
    let _ = writeln!(out, "  if k == 1 || mod(k - 1, stride) == 0");
    let _ = writeln!(out, "    printf('%.17g', ts(k));");
    let _ = writeln!(out, "    printf(',%.17g', traj(:, k));");
    let _ = writeln!(out, "    printf('\\n');");
    let _ = writeln!(out, "  end");
    let _ = writeln!(out, "end");
    out.push('\n');

    // Resolve a symbol onto its MATLAB binding: state -> state(i); parameters
    // and non-state inputs -> local variables defined at the top of the function.
    let resolve = |id: &Identifier| -> String {
        if let Some(index) = states.iter().position(|state| *state == id) {
            format!("state({})", index + 1)
        } else {
            id.as_str().to_owned()
        }
    };

    // Derivative function.
    let _ = writeln!(out, "function dstate = lawsynth_derivatives(t, state)");
    for (id, parameter) in world.parameters() {
        let _ = writeln!(out, "  {} = {};", id.as_str(), python_number(parameter.value));
    }
    for variable in world.variables().values() {
        if variable.role != VariableRole::State && !world.parameters().contains_key(&variable.id) {
            let _ = writeln!(
                out,
                "  {} = 0.0; % non-state input, edit as needed",
                variable.id.as_str()
            );
        }
    }
    let _ = writeln!(out, "  dstate = zeros({dim}, 1);");
    for (index, state) in states.iter().enumerate() {
        match world.laws().get(*state) {
            Some(law) => {
                let expression = render_matlab_expression(&law.expression, &resolve);
                let _ = writeln!(out, "  dstate({}) = {expression};", index + 1);
            }
            None => {
                let _ = writeln!(out, "  dstate({}) = 0.0;", index + 1);
            }
        }
    }
    let _ = writeln!(out, "end");
    out.push('\n');

    // RK4 integrator.
    out.push_str(
        "function [ts, traj] = lawsynth_simulate(state, t0, t1, dt)\n\
\x20 steps = round((t1 - t0) / dt);\n\
\x20 ts = zeros(1, steps + 1);\n\
\x20 traj = zeros(numel(state), steps + 1);\n\
\x20 ts(1) = t0;\n\
\x20 traj(:, 1) = state;\n\
\x20 t = t0;\n\
\x20 for s = 1:steps\n\
\x20   k1 = lawsynth_derivatives(t, state);\n\
\x20   k2 = lawsynth_derivatives(t + 0.5 * dt, state + 0.5 * dt * k1);\n\
\x20   k3 = lawsynth_derivatives(t + 0.5 * dt, state + 0.5 * dt * k2);\n\
\x20   k4 = lawsynth_derivatives(t + dt, state + dt * k3);\n\
\x20   state = state + dt * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;\n\
\x20   t = t + dt;\n\
\x20   ts(s + 1) = t;\n\
\x20   traj(:, s + 1) = state;\n\
\x20 end\n\
end\n",
    );

    out
}

// --- LaTeX ----------------------------------------------------------------

pub(crate) fn emit_latex(world: &World, name: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "% Law system for '{name}', generated by LawSynth.");
    let _ = writeln!(out, "\\begin{{align*}}");
    let laws: Vec<(&Identifier, String)> = world
        .laws()
        .iter()
        .map(|(target, law)| (target, render_latex_law(target.as_str(), &law.expression)))
        .collect();
    for (index, (_, row)) in laws.iter().enumerate() {
        let terminator = if index + 1 < laws.len() { " \\\\" } else { "" };
        let _ = writeln!(out, "    {row}{terminator}");
    }
    let _ = writeln!(out, "\\end{{align*}}");

    if !world.parameters().is_empty() {
        out.push('\n');
        let _ = writeln!(out, "% Parameters:");
        for (id, parameter) in world.parameters() {
            let _ = writeln!(
                out,
                "%   {} = {}",
                render_latex_expression(&lawsynth_expr::Expr::symbol(id.clone())),
                format_number(parameter.value)
            );
        }
    }
    out
}

// --- JSON -----------------------------------------------------------------

fn emit_json(world: &World, name: &str) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    let _ = writeln!(out, "  \"name\": {},", json_string(name));
    let _ = writeln!(out, "  \"kind\": \"continuous\",");
    let _ =
        writeln!(out, "  \"description\": \"LawSynth executable world: dx/dt = f(x; params).\",");

    // Variables.
    out.push_str("  \"variables\": [\n");
    let variables: Vec<String> = world
        .variables()
        .values()
        .map(|variable| {
            let unit = variable.unit.as_ref().map(|unit| unit.canonical().to_owned());
            format!(
                "    {{ \"id\": {}, \"role\": {}, \"unit\": {} }}",
                json_string(variable.id.as_str()),
                json_string(role_name(variable.role)),
                json_optional_string(unit.as_deref())
            )
        })
        .collect();
    out.push_str(&variables.join(",\n"));
    out.push_str("\n  ],\n");

    // Parameters.
    out.push_str("  \"parameters\": [\n");
    let parameters: Vec<String> = world
        .parameters()
        .values()
        .map(|parameter| {
            let unit = parameter.unit.as_ref().map(|unit| unit.canonical().to_owned());
            format!(
                "    {{ \"id\": {}, \"value\": {}, \"unit\": {} }}",
                json_string(parameter.id.as_str()),
                python_number(parameter.value),
                json_optional_string(unit.as_deref())
            )
        })
        .collect();
    out.push_str(&parameters.join(",\n"));
    out.push_str("\n  ],\n");

    // Laws.
    out.push_str("  \"laws\": [\n");
    let states = state_ids(world);
    let resolve = |id: &Identifier| -> String {
        if states.contains(id) {
            format!("state[{:?}]", id.as_str())
        } else {
            format!("params[{:?}]", id.as_str())
        }
    };
    let laws: Vec<String> = world
        .laws()
        .iter()
        .map(|(target, law)| {
            let reads: Vec<String> = lawsynth_world::expression_symbols(&law.expression)
                .iter()
                .map(|id| json_string(id.as_str()))
                .collect();
            format!(
                "    {{\n      \"target\": {},\n      \"derivative\": {},\n      \"equation\": {},\n      \"latex\": {},\n      \"python\": {},\n      \"reads\": [{}]\n    }}",
                json_string(target.as_str()),
                json_string(&format!("d{}/dt", target.as_str())),
                json_string(&render_expression(&law.expression)),
                json_string(&render_latex_expression(&law.expression)),
                json_string(&render_python_expression(&law.expression, &resolve)),
                reads.join(", ")
            )
        })
        .collect();
    out.push_str(&laws.join(",\n"));
    out.push_str("\n  ]\n");
    out.push_str("}\n");
    out
}

fn role_name(role: VariableRole) -> &'static str {
    match role {
        VariableRole::State => "state",
        VariableRole::Control => "control",
        VariableRole::Exogenous => "exogenous",
        VariableRole::Observed => "observed",
        VariableRole::Latent => "latent",
        VariableRole::Derived => "derived",
    }
}

/// Serializes a string as a JSON string literal with the required escapes.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for character in value.chars() {
        match character {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", control as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn json_optional_string(value: Option<&str>) -> String {
    match value {
        Some(value) => json_string(value),
        None => "null".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use lawsynth_expr::Expr;
    use lawsynth_world::{ContinuousLaw, Parameter, Variable, VariableRole, World};

    use super::*;

    fn id(value: &str) -> Identifier {
        Identifier::new(value).unwrap()
    }

    fn decay_world() -> World {
        World::new(
            [Variable::new(id("x"), VariableRole::State)],
            [Parameter::new(id("k"), 0.5)],
            [ContinuousLaw::new(
                id("x"),
                Expr::product(
                    Expr::unary(lawsynth_expr::UnaryOperator::Negate, Expr::symbol(id("k"))),
                    Expr::symbol(id("x")),
                ),
            )],
        )
        .unwrap()
    }

    #[test]
    fn python_export_binds_states_and_params() {
        let python = emit_python(&decay_world(), "decay");
        assert!(python.contains("import math"));
        assert!(python.contains("\"k\": 0.5"));
        assert!(python.contains("STATE_VARS = [\"x\"]"));
        assert!(python.contains("\"x\": -params[\"k\"] * state[\"x\"]"));
        assert!(python.contains("def simulate("));
    }

    #[test]
    fn latex_export_is_an_align_block() {
        let latex = emit_latex(&decay_world(), "decay");
        assert!(latex.contains("\\begin{align*}"));
        assert!(latex.contains("\\dot{x} &="));
        assert!(latex.contains("\\end{align*}"));
    }

    #[test]
    fn json_export_is_structured_and_escaped() {
        let json = emit_json(&decay_world(), "decay");
        assert!(json.contains("\"variables\""));
        assert!(json.contains("\"parameters\""));
        assert!(json.contains("\"target\": \"x\""));
        assert!(json.contains("\"value\": 0.5"));
    }

    /// A two-state world (dx/dt = -k*x, dy/dt = x - y) exercising positional
    /// `state[i]`/`dstate[i]` layout in the array-based emitters.
    fn two_state_world() -> World {
        World::new(
            [
                Variable::new(id("x"), VariableRole::State),
                Variable::new(id("y"), VariableRole::State),
            ],
            [Parameter::new(id("k"), 0.5)],
            [
                ContinuousLaw::new(
                    id("x"),
                    Expr::product(
                        Expr::unary(lawsynth_expr::UnaryOperator::Negate, Expr::symbol(id("k"))),
                        Expr::symbol(id("x")),
                    ),
                ),
                ContinuousLaw::new(
                    id("y"),
                    Expr::difference(Expr::symbol(id("x")), Expr::symbol(id("y"))),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn c_export_is_a_compilable_source_shape() {
        let source = emit_c(&two_state_world(), "twostate");
        assert!(source.contains("#include <math.h>"));
        assert!(source.contains("#define P_k (0.5)"));
        assert!(source.contains("enum { LAWSYNTH_STATE_DIM = 2 };"));
        assert!(
            source.contains(
                "void lawsynth_derivatives(double t, const double *state, double *dstate)"
            )
        );
        // Positional layout: x is state[0]/dstate[0], y is state[1]/dstate[1].
        assert!(source.contains("dstate[0] = -P_k * state[0];"));
        assert!(source.contains("dstate[1] = state[0] - state[1];"));
        assert!(source.contains("void lawsynth_simulate("));
        assert!(source.contains("int main(void)"));
    }

    #[test]
    fn onnx_export_is_an_honest_labeled_graph() {
        let graph = emit_onnx(&two_state_world(), "twostate");
        assert!(graph.contains("\"format\": \"lawsynth-computation-graph\""));
        assert!(graph.contains("NOT an .onnx binary"));
        assert!(graph.contains("\"onnx_op\""));
        assert!(graph.contains("\"name\": \"dx/dt\""));
        assert!(graph.contains("\"name\": \"dy/dt\""));
    }

    #[test]
    fn matlab_export_is_a_function_plus_script() {
        let m = emit_matlab(&two_state_world(), "twostate");
        assert!(m.contains("function dstate = lawsynth_derivatives(t, state)"));
        assert!(m.contains("k = 0.5;"));
        assert!(m.contains("dstate(1) = -k * state(1);"));
        assert!(m.contains("dstate(2) = state(1) - state(2);"));
        assert!(m.contains("function [ts, traj] = lawsynth_simulate("));
    }
}
