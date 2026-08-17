# Usage — scenario-exporter

## The scenario type

```rust
use lawsynth_scenario_exporter::Scenario;

let scenario = Scenario {
    id: "damped-oscillator".into(),
    variables: vec!["x".into(), "v".into()],
    initial_state: vec![1.0, 0.0],
    laws: vec!["v".into(), "-x - 0.1 * v".into()],
};
```

`variables`, `initial_state`, and `laws` are parallel: `laws[i]` is the
discovered time-derivative expression for `variables[i]`.

## Exporting

```rust
use lawsynth_scenario_exporter::{ExportFormat, ScenarioExporter};

let exporter = ScenarioExporter::new();
let json  = exporter.export(&scenario, ExportFormat::Json)?;
let world = exporter.export(&scenario, ExportFormat::World)?;
# Ok::<(), lawsynth_plugin_api::PluginError>(())
```

Each call returns an `ExportArtifact { content: String, media_type: &'static str }`.

### JSON output

```json
{
  "id": "damped-oscillator",
  "variables": ["x", "v"],
  "initial_state": [1.0, 0.0],
  "laws": ["v", "-x - 0.1 * v"]
}
```

Numbers use a full-precision representation so a re-import is lossless; strings
are escaped (`"`, `\`, control characters as `\u00XX`).

### World output

```text
# scenario: damped-oscillator
var x = 1.0
var v = 0.0
d(x)/dt = v
d(v)/dt = -x - 0.1 * v
```

This is exactly the grammar `world-validator-wasi` accepts, so an exported
scenario can be validated and re-imported without transformation.

## Validation and errors

Before serializing, `ScenarioExporter::validate` enforces:

- a non-empty, ≤255-byte, NUL-free `id`;
- non-empty and matching `variables` / `initial_state` / `laws` arities;
- unique variable names that are valid, non-reserved identifiers;
- a finite initial state;
- non-empty, NUL-free law expressions.

Any violation returns `PluginError::InvalidData`. Branch on the variant rather
than parsing the message.

## Host integration

The exporter is discovered from its `plugin.manifest` (the content of
`plugin.toml`), validated, and confirmed against the host's granted capabilities
(`artifact.write`, `dataset.read`). It runs only while the plugin lifecycle is
`Running`. The host is responsible for writing the returned `content` to its
artifact store and for enforcing `max_output_bytes`.
