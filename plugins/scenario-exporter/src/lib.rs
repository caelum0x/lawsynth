//! Scenario exporter plugin.
//!
//! Serializes a validated LawSynth [`Scenario`] — a world plus the laws
//! discovered for it — into a portable, deterministic artifact that another
//! tool can archive, diff, or re-import. Two formats are supported: canonical
//! JSON and the same line-oriented world grammar consumed by the
//! `world-validator-wasi` plugin.
//!
//! The crate is intentionally dependency-free (only the stable
//! `lawsynth-plugin-api` path dependency) so it can be built as a small WASI
//! artifact-writer loaded through `lawsynth-plugin-host`. It declares the
//! `artifact.write` and `dataset.read` capabilities (see `plugin.toml`) and
//! reports failures using [`PluginError`] so a host branches on the error
//! variant rather than parsing display strings.

use lawsynth_plugin_api::PluginError;
use std::collections::BTreeSet;

const MAX_SCENARIO_ID_BYTES: usize = 255;

/// A discovered scenario ready to be exported.
///
/// `variables`, `initial_state`, and `laws` are parallel: `laws[i]` is the
/// discovered time-derivative expression for `variables[i]`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub variables: Vec<String>,
    pub initial_state: Vec<f64>,
    pub laws: Vec<String>,
}

/// Output serialization requested by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExportFormat {
    /// Canonical, deterministically ordered JSON.
    Json,
    /// The line-oriented `var ... / d(...)/dt = ...` world grammar.
    World,
}

impl ExportFormat {
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Json => "application/json",
            Self::World => "text/plain; charset=utf-8",
        }
    }
}

/// A rendered export artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportArtifact {
    pub content: String,
    pub media_type: &'static str,
}

/// Stateless scenario serializer.
#[derive(Clone, Copy, Debug, Default)]
pub struct ScenarioExporter;

impl ScenarioExporter {
    pub const fn new() -> Self {
        Self
    }

    /// Validate and render a scenario in the requested format.
    pub fn export(
        &self,
        scenario: &Scenario,
        format: ExportFormat,
    ) -> Result<ExportArtifact, PluginError> {
        self.validate(scenario)?;
        let content = match format {
            ExportFormat::Json => render_json(scenario),
            ExportFormat::World => render_world(scenario),
        };
        Ok(ExportArtifact {
            content,
            media_type: format.media_type(),
        })
    }

    /// Structural validation applied before serialization, so an exporter never
    /// emits an artifact that would fail to re-import.
    pub fn validate(&self, scenario: &Scenario) -> Result<(), PluginError> {
        if scenario.id.is_empty()
            || scenario.id.len() > MAX_SCENARIO_ID_BYTES
            || scenario.id.contains('\0')
        {
            return Err(PluginError::InvalidData(
                "scenario id is empty, too large, or contains a NUL byte".into(),
            ));
        }
        let count = scenario.variables.len();
        if count == 0 {
            return Err(PluginError::InvalidData(
                "scenario declares no variables".into(),
            ));
        }
        if scenario.initial_state.len() != count || scenario.laws.len() != count {
            return Err(PluginError::InvalidData(format!(
                "scenario has {count} variables, {} initial values, and {} laws; all must match",
                scenario.initial_state.len(),
                scenario.laws.len()
            )));
        }
        let mut seen = BTreeSet::new();
        for name in &scenario.variables {
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
        if scenario.initial_state.iter().any(|value| !value.is_finite()) {
            return Err(PluginError::InvalidData(
                "scenario initial state must be finite".into(),
            ));
        }
        for law in &scenario.laws {
            if law.trim().is_empty() || law.contains('\0') {
                return Err(PluginError::InvalidData(
                    "scenario law is empty or contains a NUL byte".into(),
                ));
            }
        }
        Ok(())
    }
}

fn render_json(scenario: &Scenario) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"id\": {},\n", json_string(&scenario.id)));
    out.push_str("  \"variables\": [");
    push_joined(&mut out, scenario.variables.iter().map(|v| json_string(v)));
    out.push_str("],\n");
    out.push_str("  \"initial_state\": [");
    push_joined(&mut out, scenario.initial_state.iter().map(json_number));
    out.push_str("],\n");
    out.push_str("  \"laws\": [");
    push_joined(&mut out, scenario.laws.iter().map(|l| json_string(l)));
    out.push_str("]\n");
    out.push('}');
    out
}

fn render_world(scenario: &Scenario) -> String {
    let mut out = format!("# scenario: {}\n", scenario.id);
    for (name, value) in scenario.variables.iter().zip(&scenario.initial_state) {
        out.push_str(&format!("var {name} = {}\n", json_number(value)));
    }
    for (name, law) in scenario.variables.iter().zip(&scenario.laws) {
        out.push_str(&format!("d({name})/dt = {law}\n"));
    }
    out
}

fn push_joined(out: &mut String, mut items: impl Iterator<Item = String>) {
    if let Some(first) = items.next() {
        out.push_str(&first);
        for item in items {
            out.push_str(", ");
            out.push_str(&item);
        }
    }
}

/// Emit a JSON string literal with the mandatory escapes so the output always
/// parses. Control characters are escaped as `\u00XX`.
fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Render a finite `f64` as a JSON number. Callers validate finiteness first;
/// this uses a full-precision representation so a re-import is lossless.
fn json_number(value: &f64) -> String {
    format!("{value:?}")
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        && name != "t"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scenario() -> Scenario {
        Scenario {
            id: "damped-oscillator".into(),
            variables: vec!["x".into(), "v".into()],
            initial_state: vec![1.0, 0.0],
            laws: vec!["v".into(), "-x - 0.1 * v".into()],
        }
    }

    #[test]
    fn exports_deterministic_json() {
        let artifact = ScenarioExporter::new()
            .export(&scenario(), ExportFormat::Json)
            .unwrap();
        assert_eq!(artifact.media_type, "application/json");
        assert!(artifact.content.contains("\"id\": \"damped-oscillator\""));
        assert!(artifact.content.contains("\"variables\": [\"x\", \"v\"]"));
        // Serialization is a pure function of the scenario.
        let again = ScenarioExporter::new()
            .export(&scenario(), ExportFormat::Json)
            .unwrap();
        assert_eq!(artifact.content, again.content);
    }

    #[test]
    fn world_export_round_trips_shape() {
        let artifact = ScenarioExporter::new()
            .export(&scenario(), ExportFormat::World)
            .unwrap();
        assert!(artifact.content.contains("var x = 1.0"));
        assert!(artifact.content.contains("d(v)/dt = -x - 0.1 * v"));
    }

    #[test]
    fn rejects_mismatched_arity() {
        let mut bad = scenario();
        bad.laws.pop();
        assert!(matches!(
            ScenarioExporter::new().export(&bad, ExportFormat::Json),
            Err(PluginError::InvalidData(_))
        ));
    }

    #[test]
    fn escapes_json_control_characters() {
        let mut scenario = scenario();
        scenario.id = "line\nbreak".into();
        let artifact = ScenarioExporter::new()
            .export(&scenario, ExportFormat::Json)
            .unwrap();
        assert!(artifact.content.contains("line\\nbreak"));
    }
}
