# Usage — custom-operator-rust

This plugin implements the `AlgorithmPlugin` trait from
`lawsynth-plugin-api`:

```rust
pub trait AlgorithmPlugin: Send + Sync {
    fn discover(&self, request: AlgorithmRequest) -> Result<AlgorithmResponse, PluginError>;
}
```

## Request contract

An `AlgorithmRequest` carries a validated columnar dataset and the name of the
target column:

```rust
use lawsynth_plugin_api::{AlgorithmRequest, Column, DataBatch, DataSchema, ScalarType};

let request = AlgorithmRequest {
    schema: DataSchema {
        columns: vec![
            Column { name: "x".into(), scalar_type: ScalarType::Float64, nullable: false },
            Column { name: "y".into(), scalar_type: ScalarType::Float64, nullable: false },
        ],
    },
    columns: vec![
        DataBatch::Float64(vec![0.0, 1.0, 2.0, 3.0]),
        DataBatch::Float64(vec![1.0, 3.0, 5.0, 7.0]),
    ],
    target: "y".into(),
};
```

The API validates the request before your code runs (`request.validate()` is
called inside `discover`):

- the columns must match the schema arity, types, and row counts;
- the target must exist in the schema and be `Float64` or `Int64`;
- all floating-point values must be finite; text must be NUL-free.

Invalid input yields a `PluginError` variant — branch on the variant rather
than parsing the display string.

## Response contract

`LinearOperator` returns the best single-feature linear fit:

```text
equation:    d(y)/dt = 2.0000000000000000 * x
score:       -0.0            # negative mean squared error (higher is better)
diagnostics: ["mean_squared_error=0.0"]
```

- `equation` is a human-readable law referencing the target and chosen feature.
- `score` is `-MSE`; the host records its interpretation in run provenance.
- Both are validated (`AlgorithmResponse::validate`) before crossing the
  boundary: the equation must be 1..=1 MiB bytes and NUL-free, the score must be
  finite.

## Configuration

`LinearOperator` has one tunable, `minimum_variance` (default `1e-12`). Columns
whose variance falls below this threshold are skipped to avoid dividing by a
near-zero denominator. A non-finite or non-positive value is rejected with
`PluginError::InvalidData`.

```rust
let operator = LinearOperator { minimum_variance: 1e-9 };
```

## Host integration

1. The host discovers this plugin from its `plugin.manifest` (the content of
   `plugin.toml`) and validates the manifest.
2. It confirms the declared capabilities (`algorithm`, `dataset.read`) are a
   subset of its granted policy.
3. It drives the lifecycle `Discovered → Validated → Starting → Running` and
   only dispatches requests while `Running`.
4. For a `wasi` build, compile with `cargo build --target wasm32-wasi
   --release` and point `entrypoint` at the resulting `.wasm` file.

See [specs/plugin-protocol](../../../specs/plugin-protocol) for the full
protocol.
