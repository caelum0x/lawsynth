# Usage — world-validator-wasi

Two ways to use the validator: the safe embedded API, and the WASI ABI export.

## Embedded (safe) API

```rust
use lawsynth_world_validator::{WorldSpec, WorldValidator};

let spec = WorldSpec {
    variables: vec!["x".into(), "v".into()],
    initial_state: vec![1.0, 0.0],
    derivatives: vec!["v".into(), "-x - 0.1 * v".into()],
};

let report = WorldValidator::new().validate(&spec)?;
println!("{} variables, {} warnings", report.variable_count, report.warnings.len());
# Ok::<(), lawsynth_plugin_api::PluginError>(())
```

Or from the line-oriented grammar:

```rust
use lawsynth_world_validator::WorldValidator;

let world = "\
var x = 1.0
var v = 0.0
d(x)/dt = v
d(v)/dt = -x - 0.1 * v
";
let report = WorldValidator::new().validate_text(world)?;
# Ok::<(), lawsynth_plugin_api::PluginError>(())
```

Failures return a `PluginError`:

- `InvalidData` — bad shape, invalid/duplicate/reserved name, non-finite initial
  value, empty or NUL-containing derivative, malformed directive.
- `ResourceLimit` — too many variables or an oversized derivative body.

Branch on the variant, not the message text.

## WASI ABI

The module exports:

```rust
#[no_mangle]
pub unsafe extern "C" fn lawsynth_world_validate(ptr: *const u8, len: usize) -> i32;
```

Host contract:

1. Allocate `len` bytes in the module's linear memory and copy the UTF-8 world
   description there.
2. Call `lawsynth_world_validate(ptr, len)`.
3. Interpret the return code:

   | Code | Meaning |
   | ---- | ------- |
   | `0`  | structurally valid |
   | `-1` | structurally invalid |
   | `-2` | input was not valid UTF-8 |
   | `-3` | exceeded a validator resource bound |

The pointer must reference `len` initialized, readable bytes that stay valid for
the duration of the call. A null pointer returns `-1`.

## Lifecycle and host duties

The plugin participates in the standard lifecycle
(`Discovered → Validated → Starting → Running`) and only serves requests while
`Running`. The host owns WASI sandboxing, memory limits, CPU metering, and
timeouts — the manifest's declared `max_*` limits are validated by the API and
compared against host policy, but enforcement is the host's responsibility.
See [specs/plugin-protocol/resources.md](../../../specs/plugin-protocol/resources.md).
