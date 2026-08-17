# Usage — external-simulator

The plugin implements the `SimulationPlugin` trait from
`lawsynth-plugin-api`:

```rust
pub trait SimulationPlugin: Send + Sync {
    fn simulate(&self, request: SimulationRequest) -> Result<SimulationResponse, PluginError>;
}
```

## Constructing the simulator core

`LinearSimulator::new(matrix, bias, max_step)` builds an integrator for the
linear system `dx/dt = matrix · x + bias`:

```rust
use lawsynth_external_simulator::LinearSimulator;

// A damped harmonic oscillator: dx/dt = v, dv/dt = -x - 0.1 v
let simulator = LinearSimulator::new(
    vec![vec![0.0, 1.0], vec![-1.0, -0.1]],
    vec![0.0, 0.0],
    0.01, // maximum Euler step
).unwrap();
```

Construction fails with `PluginError::InvalidData` if the matrix is not square,
the bias is empty, `max_step` is not finite and positive, or any parameter is
non-finite.

## Request / response contract

```rust
use lawsynth_plugin_api::{SimulationPlugin, SimulationRequest};

let request = SimulationRequest {
    initial_state: vec![1.0, 0.0],
    times: vec![0.0, 0.5, 1.0, 1.5, 2.0], // strictly increasing, finite
};
let response = simulator.simulate(request).unwrap();
// response.states[i] is the state vector at times[i].
```

The API validates both sides:

- `SimulationRequest::validate` — non-empty finite initial state, non-empty
  finite strictly increasing times, and a point-count ceiling.
- `SimulationResponse::validate_for` — one state row per requested time, each
  row the width of the initial state, and every value finite.

Divergence to a non-finite state during integration is reported as
`PluginError::InvalidData`.

## Running as an out-of-process worker

The manifest advertises `kind = "process"`. A worker binary wraps the core:

```text
loop {
    read a length-delimited Frame from stdin
    match frame.kind {
        Hello    => reply Hello (confirm protocol version 1)
        Request  => decode SimulationRequest, run simulator, reply Response/Error
        Shutdown => drain and exit
    }
}
```

Frame encode/decode is provided by `lawsynth_plugin_api::Frame`. The host owns
spawning, sandboxing, resource metering, and timeouts
(`crates/lawsynth-plugin-host` provides `ProcessSpec`/`ProcessHandle`). Payload
serialization inside a frame is agreed out of band — the API standardizes
framing and validation, not the wire format of the payload.

## Host integration

1. Discover and validate the manifest; confirm the `simulator` capability is
   granted.
2. Spawn the entrypoint via the host's process module.
3. Drive the lifecycle to `Running`; dispatch `Request` frames.
4. Enforce `max_output_bytes` against `SimulationResponse` size — the API
   provides `estimated_output_bytes` and `validate_for_with_limits` to help.
