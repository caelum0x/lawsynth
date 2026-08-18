//! High-level operations exposed across the C-ABI boundary.
//!
//! Every operation takes a UTF-8 JSON request and returns a UTF-8 JSON payload
//! (or a [`WasmError`]). All computation is deterministic: fixed-step RK4, pure
//! expression evaluation, and byte-exact bundle codecs — no clock, RNG, or I/O.

use std::collections::BTreeMap;

use lawsynth_wasm::{
    Bundle, Event, EventDirection, Expression, MemoryBudget, WasmConfig, WasmError, World,
    simulate_rk4,
};

use crate::convert::{build_world, state_vector, trajectory_to_json};
use crate::json::{Json, parse};

/// Upper bound on the size of an incoming request, mirroring the playground's
/// default `maximumRequestBytes` (8 MiB). Enforced *before* the bytes are read
/// out of linear memory, surfacing an honest [`WasmError::MemoryLimit`].
pub const MAX_REQUEST_BYTES: usize = 8 * 1024 * 1024;

/// Reject an over-large request using the core's [`MemoryBudget`] accounting.
///
/// Returns [`WasmError::MemoryLimit`] rather than attempting (and failing) to
/// allocate the buffer, so the caller never OOMs.
pub fn check_request_size(len: usize) -> Result<(), WasmError> {
    let mut budget = MemoryBudget::new(MAX_REQUEST_BYTES)?;
    budget.reserve(len)
}

fn parse_request(input: &str) -> Result<Json, WasmError> {
    parse(input).map_err(WasmError::InvalidWorld)
}

fn required<'a>(root: &'a Json, key: &str) -> Result<&'a Json, WasmError> {
    root.get(key).ok_or_else(|| WasmError::InvalidWorld(format!("request is missing '{key}'")))
}

fn number(root: &Json, key: &str) -> Result<f64, WasmError> {
    required(root, key)?
        .as_f64()
        .ok_or_else(|| WasmError::InvalidWorld(format!("request field '{key}' must be finite")))
}

/// Simulate a world with fixed-step RK4.
///
/// Request: `{ world, initial, parameters?, start, end, step }`.
/// Response: `{ variables, times, values }` (the playground's `TrajectoryInput`).
pub fn simulate(input: &str) -> Result<String, WasmError> {
    let request = parse_request(input)?;
    let world_json = required(&request, "world")?;
    let initial = required(&request, "initial")?;
    let params = request.get("parameters");
    let start = number(&request, "start")?;
    let end = number(&request, "end")?;
    let step = number(&request, "step")?;

    let resolved = build_world(world_json, initial, params)?;
    let config = WasmConfig::default();
    let trajectory = simulate_rk4(&resolved.world, start, end, step, &config)?;

    // Faithfully account for the serialized result against the memory budget so
    // an unexpectedly large trajectory surfaces MEMORY_LIMIT instead of OOM.
    let estimated = trajectory
        .len()
        .saturating_mul(trajectory.dimension().saturating_add(1))
        .saturating_mul(24);
    let mut budget = MemoryBudget::new(config.max_memory_bytes)?;
    budget.reserve(estimated)?;

    Ok(trajectory_to_json(&resolved.state_names, &trajectory).to_json_string())
}

/// Validate that a world can be lowered to the executable core without running a
/// simulation.
///
/// Accepts either a bare world document (the playground's `validateWorld`
/// signature) or an envelope `{ world, initial?, parameters? }`. Initial state is
/// optional here — validation checks variable names, law coverage, and
/// expression lowering.
/// Response: `{ ok: true, variables, dimension }`.
pub fn validate_world(input: &str) -> Result<String, WasmError> {
    let request = parse_request(input)?;
    // A bare world has no "world" key; treat the whole document as the world.
    let world_json = request.get("world").unwrap_or(&request);
    let initial = request.get("initial");
    let params = request.get("parameters");
    let names = crate::convert::validate_world_shape(world_json, initial, params)?;
    let variables = Json::Arr(names.iter().map(|name| Json::Str(name.clone())).collect());
    let payload = Json::Obj(vec![
        ("ok".to_string(), Json::Bool(true)),
        ("variables".to_string(), variables),
        ("dimension".to_string(), Json::Num(names.len() as f64)),
    ]);
    Ok(payload.to_json_string())
}

/// Evaluate the derivative field at a single point.
///
/// Request: `{ world, parameters?, t, state }` where `state` maps each state
/// variable id to a value.
/// Response: `{ variables, derivative }`.
pub fn derivative(input: &str) -> Result<String, WasmError> {
    let request = parse_request(input)?;
    let world_json = required(&request, "world")?;
    let state = required(&request, "state")?;
    let params = request.get("parameters");
    let time = number(&request, "t")?;

    // The state point doubles as the "initial" map for world construction.
    let resolved = build_world(world_json, state, params)?;
    let vector = state_vector(&resolved.state_names, state)?;
    let derivative = resolved.world.derivative_at(time, &vector)?;

    let variables =
        Json::Arr(resolved.state_names.iter().map(|name| Json::Str(name.clone())).collect());
    let values = Json::Arr(derivative.into_iter().map(Json::Num).collect());
    let payload =
        Json::Obj(vec![("variables".to_string(), variables), ("derivative".to_string(), values)]);
    Ok(payload.to_json_string())
}

/// Parse and evaluate a single scalar expression at a scope.
///
/// Request: `{ expression: "sin(x)+1", scope: { x: 0.5 } }`.
/// Response: `{ value }`.
pub fn eval_expression(input: &str) -> Result<String, WasmError> {
    let request = parse_request(input)?;
    let source = required(&request, "expression")?
        .as_str()
        .ok_or_else(|| WasmError::InvalidExpression("'expression' must be a string".into()))?;
    let expression = Expression::parse(source)?;

    let mut scope = BTreeMap::new();
    if let Some(Json::Obj(entries)) = request.get("scope") {
        for (key, value) in entries {
            let number = value.as_f64().ok_or_else(|| {
                WasmError::InvalidExpression(format!("scope value {key} is not finite"))
            })?;
            scope.insert(key.clone(), number);
        }
    }
    let value = expression.evaluate(&scope)?;
    let payload = Json::Obj(vec![("value".to_string(), Json::Num(value))]);
    Ok(payload.to_json_string())
}

/// Encode a world (plus optional events) into the native `.lsworld` bundle
/// format, returning the raw bytes.
///
/// Request: `{ world, initial, parameters?, events? }`. The returned payload is
/// binary (not UTF-8) — the glue surfaces it as a `Uint8Array`.
pub fn bundle_encode(input: &str) -> Result<Vec<u8>, WasmError> {
    let request = parse_request(input)?;
    let world_json = required(&request, "world")?;
    let initial = required(&request, "initial")?;
    let params = request.get("parameters");
    let resolved = build_world(world_json, initial, params)?;
    let events = build_events(&resolved.world, world_json, params)?;
    let bundle = Bundle::new(resolved.world, events)?;
    bundle.encode()
}

/// Decode a native `.lsworld` bundle back into a compact JSON description.
///
/// Input: raw bundle bytes. Response: `{ variables, initial, derivatives, events }`.
pub fn bundle_decode(bytes: &[u8]) -> Result<String, WasmError> {
    let bundle = Bundle::decode(bytes)?;
    let world = &bundle.world;
    let variables = Json::Arr(world.variables.iter().map(|name| Json::Str(name.clone())).collect());
    let initial = Json::Obj(
        world
            .variables
            .iter()
            .zip(&world.initial_state)
            .map(|(name, value)| (name.clone(), Json::Num(*value)))
            .collect(),
    );
    let derivatives =
        Json::Arr(world.derivatives.iter().map(|expr| Json::Str(expr.source())).collect());
    let events = Json::Arr(
        bundle
            .events
            .iter()
            .map(|event| {
                Json::Obj(vec![
                    ("name".to_string(), Json::Str(event.name.clone())),
                    (
                        "direction".to_string(),
                        Json::Str(direction_label(event.direction).to_string()),
                    ),
                    ("condition".to_string(), Json::Str(event.condition.source())),
                ])
            })
            .collect(),
    );
    let payload = Json::Obj(vec![
        ("variables".to_string(), variables),
        ("initial".to_string(), initial),
        ("derivatives".to_string(), derivatives),
        ("events".to_string(), events),
    ]);
    Ok(payload.to_json_string())
}

fn direction_label(direction: EventDirection) -> &'static str {
    match direction {
        EventDirection::Any => "any",
        EventDirection::Rising => "rising",
        EventDirection::Falling => "falling",
    }
}

fn build_events(
    world: &World,
    world_json: &Json,
    params_override: Option<&Json>,
) -> Result<Vec<Event>, WasmError> {
    let mut events = Vec::new();
    let Some(list) = world_json.get("events").and_then(Json::as_array) else {
        return Ok(events);
    };
    let states: std::collections::BTreeSet<String> = world.variables.iter().cloned().collect();
    let time = world_json
        .get("time")
        .and_then(|time| time.get("symbol"))
        .and_then(Json::as_str)
        .unwrap_or("t")
        .to_string();
    let params = crate::convert::parameter_map_public(world_json, params_override)?;
    for event in list {
        let id = event
            .get("id")
            .and_then(Json::as_str)
            .ok_or_else(|| WasmError::InvalidBundle("event needs a string id".into()))?;
        let condition = event
            .get("condition")
            .ok_or_else(|| WasmError::InvalidBundle("event needs a condition".into()))?;
        let expression =
            crate::convert::lower_expression_public(condition, &params, &time, &states)?;
        let direction = match event.get("direction").and_then(Json::as_str) {
            Some("rising") => EventDirection::Rising,
            Some("falling") => EventDirection::Falling,
            _ => EventDirection::Any,
        };
        events.push(Event::new(id, expression, direction)?);
    }
    Ok(events)
}
