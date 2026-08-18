//! Translation between the playground's JSON documents and the native
//! `lawsynth-wasm` domain types.
//!
//! The playground serializes a rich `WorldDefinition` (structured expression
//! ASTs, parameters, laws). The executable core is a scalar ODE evaluator over a
//! flat state vector. This module bridges the two deterministically: it selects
//! `state` variables, resolves parameter symbols to constants, and lowers the
//! structured expression AST into the core [`Expression`] enum.

use std::collections::{BTreeMap, BTreeSet};

use lawsynth_wasm::{BinaryOp, Expression, Function, Trajectory, WasmError, World};

use crate::json::Json;

/// A fully resolved world plus the ordered state-variable names, ready to drive
/// the native simulator.
pub struct ResolvedWorld {
    pub world: World,
    pub state_names: Vec<String>,
}

/// Default identifier the core uses for the time variable inside `state_map`.
const CORE_TIME_SYMBOL: &str = "t";

/// Extract the ordered list of `state` variable identifiers from a world object.
fn state_variable_ids(world: &Json) -> Result<Vec<String>, WasmError> {
    let variables = world
        .get("variables")
        .and_then(Json::as_array)
        .ok_or_else(|| WasmError::InvalidWorld("world.variables must be an array".into()))?;
    let mut ids = Vec::new();
    for variable in variables {
        let role = variable.get("role").and_then(Json::as_str).unwrap_or("state");
        if role != "state" {
            continue;
        }
        let id = variable
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| WasmError::InvalidWorld("variable is missing a string id".into()))?;
        ids.push(id.to_string());
    }
    if ids.is_empty() {
        return Err(WasmError::InvalidWorld("world has no state variables".into()));
    }
    Ok(ids)
}

/// Build the parameter substitution map: world defaults overridden by the
/// request's `parameters` block.
fn parameter_map(
    world: &Json,
    overrides: Option<&Json>,
) -> Result<BTreeMap<String, f64>, WasmError> {
    let mut params = BTreeMap::new();
    if let Some(list) = world.get("parameters").and_then(Json::as_array) {
        for parameter in list {
            let id = parameter.get("id").and_then(Json::as_str).ok_or_else(|| {
                WasmError::InvalidWorld("parameter is missing a string id".into())
            })?;
            let value = parameter.get("value").and_then(Json::as_f64).ok_or_else(|| {
                WasmError::InvalidWorld(format!("parameter {id} has no finite value"))
            })?;
            params.insert(id.to_string(), value);
        }
    }
    if let Some(Json::Obj(entries)) = overrides {
        for (key, value) in entries {
            let number = value.as_f64().ok_or_else(|| {
                WasmError::InvalidWorld(format!("parameter override {key} is not finite"))
            })?;
            params.insert(key.clone(), number);
        }
    }
    Ok(params)
}

/// Lower a structured expression AST node into the core [`Expression`] enum,
/// resolving parameter symbols to constants.
fn lower_expression(
    node: &Json,
    params: &BTreeMap<String, f64>,
    time_symbol: &str,
    states: &BTreeSet<String>,
) -> Result<Expression, WasmError> {
    let kind = node
        .get("kind")
        .and_then(Json::as_str)
        .ok_or_else(|| WasmError::InvalidExpression("expression node needs a kind".into()))?;
    match kind {
        "constant" => {
            let value = node.get("value").and_then(Json::as_f64).ok_or_else(|| {
                WasmError::InvalidExpression("constant must hold a finite number".into())
            })?;
            Ok(Expression::Constant(value))
        }
        "symbol" => {
            let id = node.get("id").and_then(Json::as_str).ok_or_else(|| {
                WasmError::InvalidExpression("symbol node needs a string id".into())
            })?;
            if let Some(value) = params.get(id) {
                Ok(Expression::Constant(*value))
            } else if id == time_symbol {
                Ok(Expression::Variable(CORE_TIME_SYMBOL.to_string()))
            } else if states.contains(id) {
                Ok(Expression::Variable(id.to_string()))
            } else {
                Err(WasmError::InvalidExpression(format!("unknown symbol {id}")))
            }
        }
        "unary" => {
            let operand = node.get("operand").ok_or_else(|| {
                WasmError::InvalidExpression("unary node needs an operand".into())
            })?;
            let inner = lower_expression(operand, params, time_symbol, states)?;
            let operator = node
                .get("operator")
                .and_then(Json::as_str)
                .ok_or_else(|| WasmError::InvalidExpression("unary node needs operator".into()))?;
            lower_unary(operator, inner)
        }
        "binary" => {
            let left_node = node
                .get("left")
                .ok_or_else(|| WasmError::InvalidExpression("binary needs left".into()))?;
            let right_node = node
                .get("right")
                .ok_or_else(|| WasmError::InvalidExpression("binary needs right".into()))?;
            let left = lower_expression(left_node, params, time_symbol, states)?;
            let right = lower_expression(right_node, params, time_symbol, states)?;
            let operator = node
                .get("operator")
                .and_then(Json::as_str)
                .ok_or_else(|| WasmError::InvalidExpression("binary needs operator".into()))?;
            lower_binary(operator, left, right)
        }
        other => Err(WasmError::Unsupported(format!(
            "expression kind '{other}' is not supported by the scalar core"
        ))),
    }
}

fn lower_unary(operator: &str, inner: Expression) -> Result<Expression, WasmError> {
    let function = match operator {
        "neg" => return Ok(Expression::Neg(Box::new(inner))),
        // tan is not a native core function; synthesize sin/cos.
        "tan" => {
            let sin =
                Expression::Function { name: Function::Sin, argument: Box::new(inner.clone()) };
            let cos = Expression::Function { name: Function::Cos, argument: Box::new(inner) };
            return Ok(Expression::Binary {
                op: BinaryOp::Divide,
                left: Box::new(sin),
                right: Box::new(cos),
            });
        }
        "abs" => Function::Abs,
        "exp" => Function::Exp,
        "log" => Function::Log,
        "sqrt" => Function::Sqrt,
        "sin" => Function::Sin,
        "cos" => Function::Cos,
        other => {
            return Err(WasmError::Unsupported(format!("unary operator '{other}' is unsupported")));
        }
    };
    Ok(Expression::Function { name: function, argument: Box::new(inner) })
}

fn lower_binary(
    operator: &str,
    left: Expression,
    right: Expression,
) -> Result<Expression, WasmError> {
    let op = match operator {
        "add" => BinaryOp::Add,
        "sub" => BinaryOp::Subtract,
        "mul" => BinaryOp::Multiply,
        "div" => BinaryOp::Divide,
        "pow" => BinaryOp::Power,
        other => {
            return Err(WasmError::Unsupported(format!(
                "binary operator '{other}' is unsupported by the scalar core"
            )));
        }
    };
    Ok(Expression::Binary { op, left: Box::new(left), right: Box::new(right) })
}

/// Find the continuous law whose `target` is `variable` and lower its expression.
fn derivative_for(
    variable: &str,
    world: &Json,
    params: &BTreeMap<String, f64>,
    time_symbol: &str,
    states: &BTreeSet<String>,
) -> Result<Expression, WasmError> {
    let laws = world
        .get("laws")
        .and_then(Json::as_array)
        .ok_or_else(|| WasmError::InvalidWorld("world.laws must be an array".into()))?;
    let mut found: Option<&Json> = None;
    for law in laws {
        let kind = law.get("kind").and_then(Json::as_str).unwrap_or("");
        let target = law.get("target").and_then(Json::as_str).unwrap_or("");
        let enabled = !matches!(law.get("enabled"), Some(Json::Bool(false)));
        if kind == "continuous" && target == variable && enabled {
            if found.is_some() {
                return Err(WasmError::InvalidWorld(format!(
                    "multiple continuous laws target {variable}"
                )));
            }
            found = Some(law);
        }
    }
    let law = found.ok_or_else(|| {
        WasmError::InvalidWorld(format!("no continuous law defines the derivative of {variable}"))
    })?;
    let expression = law
        .get("expression")
        .ok_or_else(|| WasmError::InvalidWorld(format!("law for {variable} has no expression")))?;
    lower_expression(expression, params, time_symbol, states)
}

/// Read the time symbol declared by the world (defaults to `t`).
fn time_symbol(world: &Json) -> String {
    world
        .get("time")
        .and_then(|time| time.get("symbol"))
        .and_then(Json::as_str)
        .unwrap_or(CORE_TIME_SYMBOL)
        .to_string()
}

/// Build a [`World`] from a world document plus an initial-state map.
pub fn build_world(
    world_json: &Json,
    initial: &Json,
    params_override: Option<&Json>,
) -> Result<ResolvedWorld, WasmError> {
    let state_names = state_variable_ids(world_json)?;
    let params = parameter_map(world_json, params_override)?;
    let time = time_symbol(world_json);
    let states: BTreeSet<String> = state_names.iter().cloned().collect();

    let mut initial_state = Vec::with_capacity(state_names.len());
    for name in &state_names {
        let value = initial.get(name).and_then(Json::as_f64).ok_or_else(|| {
            WasmError::InvalidWorld(format!("initial state is missing a finite value for {name}"))
        })?;
        initial_state.push(value);
    }

    let mut derivatives = Vec::with_capacity(state_names.len());
    for name in &state_names {
        derivatives.push(derivative_for(name, world_json, &params, &time, &states)?);
    }

    let world = World::new(state_names.clone(), initial_state, derivatives)?;
    Ok(ResolvedWorld { world, state_names })
}

/// Validate that a world lowers to the executable core, WITHOUT requiring a
/// full initial-state map. Missing state values default to `0.0` — this checks
/// variable names, law coverage, and expression lowering only, matching the
/// playground's `validateWorld(worldJson)` signature (a bare world document).
pub fn validate_world_shape(
    world_json: &Json,
    initial: Option<&Json>,
    params_override: Option<&Json>,
) -> Result<Vec<String>, WasmError> {
    let state_names = state_variable_ids(world_json)?;
    let params = parameter_map(world_json, params_override)?;
    let time = time_symbol(world_json);
    let states: BTreeSet<String> = state_names.iter().cloned().collect();

    let initial_state: Vec<f64> = state_names
        .iter()
        .map(|name| initial.and_then(|map| map.get(name)).and_then(Json::as_f64).unwrap_or(0.0))
        .collect();

    let mut derivatives = Vec::with_capacity(state_names.len());
    for name in &state_names {
        derivatives.push(derivative_for(name, world_json, &params, &time, &states)?);
    }
    World::new(state_names.clone(), initial_state, derivatives)?;
    Ok(state_names)
}

/// Serialize a native [`Trajectory`] into the playground's `TrajectoryInput`
/// shape: `{ variables, times, values }`.
pub fn trajectory_to_json(names: &[String], trajectory: &Trajectory) -> Json {
    let variables = Json::Arr(names.iter().map(|name| Json::Str(name.clone())).collect());
    let times = Json::Arr(trajectory.times.iter().map(|time| Json::Num(*time)).collect());
    let values = Json::Arr(
        trajectory
            .values
            .iter()
            .map(|row| Json::Arr(row.iter().map(|value| Json::Num(*value)).collect()))
            .collect(),
    );
    Json::Obj(vec![
        ("variables".to_string(), variables),
        ("times".to_string(), times),
        ("values".to_string(), values),
    ])
}

/// Public wrapper: resolve a world's parameter substitution map (used by the
/// bundle encoder to lower event-condition expressions).
pub fn parameter_map_public(
    world: &Json,
    overrides: Option<&Json>,
) -> Result<BTreeMap<String, f64>, WasmError> {
    parameter_map(world, overrides)
}

/// Public wrapper: lower a structured expression AST into the core enum.
pub fn lower_expression_public(
    node: &Json,
    params: &BTreeMap<String, f64>,
    time_symbol: &str,
    states: &BTreeSet<String>,
) -> Result<Expression, WasmError> {
    lower_expression(node, params, time_symbol, states)
}

/// Build a point `state_map`-style vector (ordered by `names`) from a JSON map.
pub fn state_vector(names: &[String], state: &Json) -> Result<Vec<f64>, WasmError> {
    let mut vector = Vec::with_capacity(names.len());
    for name in names {
        let value = state.get(name).and_then(Json::as_f64).ok_or_else(|| {
            WasmError::InvalidWorld(format!("state point is missing a finite value for {name}"))
        })?;
        vector.push(value);
    }
    Ok(vector)
}
