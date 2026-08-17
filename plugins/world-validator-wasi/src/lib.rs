//! WASI world validator plugin.
//!
//! This crate validates the *structure* of a LawSynth world before a host lets
//! it enter a run. It mirrors the structural invariants enforced by
//! `lawsynth_wasm::World` (matching variable / state / derivative arities,
//! unique well-formed variable names, finite initial state) but stays
//! dependency-free so it can be compiled to a small `wasm32-wasi` component and
//! loaded through `lawsynth-plugin-host`. Numeric evaluation of the derivative
//! expressions is intentionally left to the host's expression engine; this
//! plugin only guarantees that a world is well-formed enough to evaluate.
//!
//! The plugin declares the `world.validate` capability (see `plugin.toml`) and
//! reports failures using [`PluginError`] from the stable plugin API so a host
//! can branch on the error variant rather than parsing display strings.

use lawsynth_plugin_api::PluginError;
use std::collections::{BTreeMap, BTreeSet};

/// A parsed, not-yet-validated world description.
///
/// The three vectors are parallel: `variables[i]` has initial value
/// `initial_state[i]` and its time derivative is described by `derivatives[i]`.
/// Derivative bodies are kept as opaque text because expression semantics are a
/// host concern; only their presence and byte-safety are checked here.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldSpec {
    pub variables: Vec<String>,
    pub initial_state: Vec<f64>,
    pub derivatives: Vec<String>,
}

/// Outcome of a successful structural validation.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WorldReport {
    pub variable_count: usize,
    pub warnings: Vec<String>,
}

/// The maximum world size this validator will accept. A host still enforces its
/// own resource limits; this bound only protects the validator itself from a
/// pathological manifest.
const MAX_VARIABLES: usize = 4_096;
const MAX_DERIVATIVE_BYTES: usize = 64 * 1024;

/// Structural validator for LawSynth worlds.
#[derive(Clone, Copy, Debug, Default)]
pub struct WorldValidator;

impl WorldValidator {
    pub const fn new() -> Self {
        Self
    }

    /// Validate a fully parsed [`WorldSpec`].
    ///
    /// Enforces the same shape invariants as the runtime world type: non-empty
    /// and matching arities, unique variable names that are valid identifiers
    /// and are not the reserved time symbol `t`, finite initial state, and
    /// non-empty NUL-free derivative bodies.
    pub fn validate(&self, spec: &WorldSpec) -> Result<WorldReport, PluginError> {
        let count = spec.variables.len();
        if count == 0 {
            return Err(PluginError::InvalidData(
                "world declares no variables".into(),
            ));
        }
        if count > MAX_VARIABLES {
            return Err(PluginError::ResourceLimit(format!(
                "world declares {count} variables, validator limit is {MAX_VARIABLES}"
            )));
        }
        if spec.initial_state.len() != count || spec.derivatives.len() != count {
            return Err(PluginError::InvalidData(format!(
                "world has {count} variables, {} initial values, and {} derivatives; all must match",
                spec.initial_state.len(),
                spec.derivatives.len()
            )));
        }

        let mut seen = BTreeSet::new();
        for name in &spec.variables {
            if !valid_identifier(name) {
                return Err(PluginError::InvalidData(format!(
                    "variable name {name:?} is not a valid, non-reserved identifier"
                )));
            }
            if !seen.insert(name.as_str()) {
                return Err(PluginError::InvalidData(format!(
                    "duplicate variable {name:?}"
                )));
            }
        }

        for (index, value) in spec.initial_state.iter().enumerate() {
            if !value.is_finite() {
                return Err(PluginError::InvalidData(format!(
                    "initial value for {:?} is not finite",
                    spec.variables[index]
                )));
            }
        }

        let mut warnings = Vec::new();
        for (index, body) in spec.derivatives.iter().enumerate() {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                return Err(PluginError::InvalidData(format!(
                    "derivative for {:?} is empty",
                    spec.variables[index]
                )));
            }
            if body.len() > MAX_DERIVATIVE_BYTES {
                return Err(PluginError::ResourceLimit(format!(
                    "derivative for {:?} exceeds {MAX_DERIVATIVE_BYTES} bytes",
                    spec.variables[index]
                )));
            }
            if body.contains('\0') {
                return Err(PluginError::InvalidData(format!(
                    "derivative for {:?} contains a NUL byte",
                    spec.variables[index]
                )));
            }
            if !mentions_any_variable(body, &spec.variables) && !body.contains('t') {
                warnings.push(format!(
                    "derivative for {:?} references neither a state variable nor time",
                    spec.variables[index]
                ));
            }
        }

        Ok(WorldReport {
            variable_count: count,
            warnings,
        })
    }

    /// Parse and validate a world from the plugin's line-oriented description
    /// grammar. Each non-empty, non-comment line is one of:
    ///
    /// ```text
    /// var <name> = <initial_value>
    /// d(<name>)/dt = <expression text>
    /// ```
    ///
    /// Lines are order-independent; every declared variable must have exactly
    /// one `var` line and one derivative line.
    pub fn validate_text(&self, text: &str) -> Result<WorldReport, PluginError> {
        let spec = parse_world(text)?;
        self.validate(&spec)
    }
}

fn parse_world(text: &str) -> Result<WorldSpec, PluginError> {
    // Preserve declaration order of each `var` line so reports are deterministic
    // regardless of where the matching derivative line appears.
    let mut order: Vec<String> = Vec::new();
    let mut values: BTreeMap<String, f64> = BTreeMap::new();
    let mut derivs: BTreeMap<String, String> = BTreeMap::new();

    for raw in text.lines() {
        let line = raw.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("var ") {
            let (name, value) = rest.split_once('=').ok_or_else(|| {
                PluginError::InvalidData(format!("expected `var <name> = <value>`: {line}"))
            })?;
            let name = name.trim().to_owned();
            let value = value.trim().parse::<f64>().map_err(|_| {
                PluginError::InvalidData(format!("initial value for {name:?} is not a number"))
            })?;
            if values.insert(name.clone(), value).is_some() {
                return Err(PluginError::InvalidData(format!(
                    "variable {name:?} declared more than once"
                )));
            }
            order.push(name);
        } else if let Some(rest) = line.strip_prefix("d(") {
            let (name, body) = rest.split_once(')').ok_or_else(|| {
                PluginError::InvalidData(format!("malformed derivative line: {line}"))
            })?;
            let body = body
                .trim_start()
                .strip_prefix("/dt")
                .and_then(|b| b.trim_start().strip_prefix('='))
                .ok_or_else(|| {
                    PluginError::InvalidData(format!("expected `d(<name>)/dt = ...`: {line}"))
                })?;
            let name = name.trim().to_owned();
            if derivs.insert(name.clone(), body.trim().to_owned()).is_some() {
                return Err(PluginError::InvalidData(format!(
                    "derivative for {name:?} declared more than once"
                )));
            }
        } else {
            return Err(PluginError::InvalidData(format!(
                "unrecognized world directive: {line}"
            )));
        }
    }

    let mut variables = Vec::with_capacity(order.len());
    let mut initial_state = Vec::with_capacity(order.len());
    let mut derivatives = Vec::with_capacity(order.len());
    for name in order {
        let value = values.get(&name).copied().expect("declared above");
        let body = derivs.remove(&name).ok_or_else(|| {
            PluginError::InvalidData(format!("variable {name:?} is missing a derivative"))
        })?;
        variables.push(name);
        initial_state.push(value);
        derivatives.push(body);
    }
    if let Some((orphan, _)) = derivs.into_iter().next() {
        return Err(PluginError::InvalidData(format!(
            "derivative references undeclared variable {orphan:?}"
        )));
    }

    Ok(WorldSpec {
        variables,
        initial_state,
        derivatives,
    })
}

/// A valid identifier starts with an ASCII letter or `_`, continues with ASCII
/// alphanumerics or `_`, and is never the reserved time symbol `t`.
fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "t"
}

fn mentions_any_variable(body: &str, variables: &[String]) -> bool {
    variables.iter().any(|name| body.contains(name.as_str()))
}

/// WASI ABI entrypoint.
///
/// A host writes the UTF-8 world description into the module's linear memory and
/// calls this export with a `(pointer, length)` pair. The return code is `0`
/// when the world is structurally valid and a negative code otherwise. Richer
/// diagnostics are available through the safe [`WorldValidator::validate_text`]
/// API when the validator is embedded directly (e.g. in tests or a trusted
/// host).
///
/// # Safety
///
/// The caller must guarantee that `ptr` points to `len` initialized, readable
/// bytes that remain valid for the duration of the call, as specified by the
/// plugin ABI documented in `docs/usage.md`. A null `ptr` is rejected.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn lawsynth_world_validate(ptr: *const u8, len: usize) -> i32 {
    if ptr.is_null() {
        return -1;
    }
    // SAFETY: the host contract (see docs/usage.md) guarantees `ptr..ptr+len`
    // is a valid, initialized, readable region for the duration of this call.
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
    let Ok(text) = std::str::from_utf8(bytes) else {
        return -2;
    };
    match WorldValidator::new().validate_text(text) {
        Ok(_) => 0,
        Err(PluginError::ResourceLimit(_)) => -3,
        Err(_) => -1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec() -> WorldSpec {
        WorldSpec {
            variables: vec!["x".into(), "y".into()],
            initial_state: vec![1.0, 0.0],
            derivatives: vec!["-x + t".into(), "x".into()],
        }
    }

    #[test]
    fn accepts_well_formed_world() {
        let report = WorldValidator::new().validate(&spec()).unwrap();
        assert_eq!(report.variable_count, 2);
        assert!(report.warnings.is_empty());
    }

    #[test]
    fn rejects_reserved_variable_name() {
        let mut bad = spec();
        bad.variables[0] = "t".into();
        assert!(matches!(
            WorldValidator::new().validate(&bad),
            Err(PluginError::InvalidData(_))
        ));
    }

    #[test]
    fn rejects_arity_mismatch() {
        let mut bad = spec();
        bad.derivatives.pop();
        assert!(matches!(
            WorldValidator::new().validate(&bad),
            Err(PluginError::InvalidData(_))
        ));
    }

    #[test]
    fn parses_and_validates_text() {
        let text = "var x = 1.0\nd(x)/dt = -x + t\n";
        let report = WorldValidator::new().validate_text(text).unwrap();
        assert_eq!(report.variable_count, 1);
    }

    #[test]
    fn warns_on_constant_derivative() {
        let spec = WorldSpec {
            variables: vec!["x".into()],
            initial_state: vec![0.0],
            derivatives: vec!["42".into()],
        };
        let report = WorldValidator::new().validate(&spec).unwrap();
        assert_eq!(report.warnings.len(), 1);
    }
}
