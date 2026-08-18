# lawsynth-wasm-bindings

A hand-rolled **C-ABI (wasm32) bindings layer** over
[`lawsynth-wasm`](../lawsynth-wasm) for the browser
[playground](../../apps/playground). It exposes a tiny `#[no_mangle] extern "C"`
surface — a `Vec`-backed linear-memory allocator plus string-in / string-out
entry points — that JavaScript drives directly.

**By design this crate uses NO `wasm-bindgen` and NO external crates.** That
keeps the build fully offline and the `.wasm` lean. The crate compiles as a
`cdylib` (the deployable `.wasm`) **and** as an `rlib`, so the exported functions
are unit-tested on the host toolchain across the same C-ABI a JS caller uses.

Everything is deterministic: fixed-step RK4, pure expression evaluation, and
byte-exact bundle codecs. No wall-clock, no RNG, no filesystem, no network.

---

## Memory & packing protocol

JavaScript drives the module entirely through raw pointers into WASM linear
memory:

1. `ls_alloc(len) -> ptr` — allocate `len` zeroed, writable bytes.
2. JS copies a UTF-8 (or, for `ls_bundle_decode`, binary) request into `ptr`.
3. JS calls an entry point `entry(ptr, len) -> resultPtr`. The **result buffer**
   is laid out as:

   ```text
   byte 0..4    u32 little-endian  N        payload length in bytes
   byte 4       u8                 status   0 = OK, 1 = ERROR
   byte 5..5+N  payload bytes
   ```

   - On **OK**: the payload is the operation's result (a bare `TrajectoryInput`
     JSON for `ls_simulate`, etc.), or raw binary for `ls_bundle_encode`.
   - On **ERROR**: the payload is JSON `{ "code": "...", "message": "..." }`.
4. JS reads `N` and `status`, copies the payload out, then frees the result with
   `ls_free(resultPtr, 5 + N)`.
5. JS frees its own input buffer with `ls_free(inputPtr, inputLen)`.

A stable machine-readable code for the most recent call is **also** exposed
out-of-band via `ls_last_error() -> *const u8` and `ls_last_error_len() -> usize`
(empty when the last call succeeded; valid until the next binding call).

`ls_result_len(ptr) -> usize` reads the 4-byte length prefix if a caller prefers
that over a `DataView`.

> **Allocator invariant:** buffers are `vec![0u8; len]` / `Vec::with_capacity`
> sized so `capacity == len`, and every `ls_free` must pass the exact length the
> region was created with (`5 + N` for result buffers). This keeps the internal
> `Vec::from_raw_parts` reclamation sound.

---

## Exports (C-ABI)

| Symbol | Signature | Purpose |
| --- | --- | --- |
| `ls_alloc` | `extern "C" fn(len: usize) -> *mut u8` | Allocate input bytes. |
| `ls_free` | `unsafe extern "C" fn(ptr: *mut u8, len: usize)` | Free an alloc/result buffer. |
| `ls_result_len` | `unsafe extern "C" fn(ptr: *const u8) -> usize` | Read a result's payload length. |
| `ls_last_error` | `extern "C" fn() -> *const u8` | Pointer to last error code bytes. |
| `ls_last_error_len` | `extern "C" fn() -> usize` | Length of last error code. |
| `ls_version` | `extern "C" fn() -> *mut u8` | Crate version, packed OK result. |
| `ls_simulate` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | Simulate a world (RK4). |
| `ls_validate_world` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | Validate a world without running. |
| `ls_derivative` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | Evaluate the derivative field at a point. |
| `ls_eval_expression` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | Parse + evaluate one scalar expression. |
| `ls_bundle_encode` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | World JSON → `.lsworld` bundle bytes. |
| `ls_bundle_decode` | `unsafe extern "C" fn(ptr, len) -> *mut u8` | Bundle bytes → JSON description. |

### Request / response JSON

**`ls_simulate`** — matches the playground's `WasmSimulationRequest` verbatim:

```jsonc
// request
{ "world": <WorldDefinition>, "initial": { "x": 1 },
  "parameters": { "k": 2 },            // optional overrides
  "start": 0, "end": 10, "step": 0.01 }
// response (playground TrajectoryInput)
{ "variables": ["x", "v"], "times": [0, 0.01, ...], "values": [[1,0], ...] }
```

**`ls_validate_world`** — accepts a bare `WorldDefinition` **or** an envelope
`{ world, initial?, parameters? }`. Response: `{ "ok": true, "variables": [...], "dimension": N }`.

**`ls_derivative`** — `{ world, parameters?, t, state: { x: 2 } }` →
`{ "variables": [...], "derivative": [...] }`.

**`ls_eval_expression`** — `{ "expression": "sin(x)+1", "scope": { "x": 0.5 } }` →
`{ "value": 1.479... }`.

**`ls_bundle_encode`** — `{ world, initial, parameters?, events? }` → OK payload is
**binary** bundle bytes (magic `LSWASM`). **`ls_bundle_decode`** — binary bundle
bytes → `{ variables, initial, derivatives, events }`.

### World lowering

The playground's rich `WorldDefinition` is lowered to the scalar ODE core:

- **State selection**: variables with `role == "state"`, in declaration order.
- **Derivatives**: the `continuous` law whose `target` matches each state var.
- **Parameters**: `symbol` nodes referencing a parameter id are folded to
  constants (world defaults, overridden by the request's `parameters`).
- **Time symbol**: `world.time.symbol` maps to the core `t`.
- **Expression subset** (native): `constant`, `symbol`, `unary`
  (`neg`/`abs`/`exp`/`log`/`sqrt`/`sin`/`cos`; `tan` synthesized as `sin/cos`),
  `binary` (`add`/`sub`/`mul`/`div`/`pow`). Anything else (`min`/`max`,
  `comparison`, `logical`, `not`, `delay`, `piecewise`, `call`) returns
  `UNSUPPORTED` rather than guessing.

### Error codes

Mapped from `lawsynth_wasm::WasmError` via `errors::code`:
`INVALID_WORLD`, `INVALID_EXPRESSION`, `INVALID_BUNDLE`, `INVALID_TRAJECTORY`,
`SIMULATION_FAILED`, `MEMORY_LIMIT`, `UNSUPPORTED`.

Requests larger than `MAX_REQUEST_BYTES` (8 MiB, mirroring the playground's
`maximumRequestBytes`) are rejected with `MEMORY_LIMIT` **before** the buffer is
dereferenced, so the module never OOMs on a hostile length.

---

## JavaScript glue

[`web/lawsynth_wasm.mjs`](web/lawsynth_wasm.mjs) is a zero-dependency ES module
that instantiates the `.wasm` and wraps the protocol into the
`LawSynthWasmBindings` contract the playground's
[`wasm.ts`](../../apps/playground/src/wasm.ts) expects:

```js
import { createBindings } from "./lawsynth_wasm.mjs";

const bindings = await createBindings(fetch("./lawsynth_wasm_bindings.wasm"));
// Wire directly into the playground's WasmRuntime:
const runtime = new WasmRuntime({ loader: () => bindings });

bindings.version();                 // "0.1.0"
bindings.simulate(requestJson);     // TrajectoryInput JSON string
bindings.validateWorld(worldJson);  // { ok, variables, dimension } JSON
bindings.memoryBytes();             // exports.memory.buffer.byteLength
```

`createBindings` accepts an `Instance`, a `Module`, a `Response`/`Promise`, raw
bytes, or a URL string. It re-fetches the memory view after each `alloc`/entry
call (WASM memory growth detaches the old `ArrayBuffer`), copies payloads out
before freeing, and throws `LawSynthWasmError` (with `.code`) on error status.

### Playground contract mapping

| Playground `LawSynthWasmBindings` member | Glue → C-ABI |
| --- | --- |
| `version()` | `ls_version` |
| `simulate(requestJson)` | `ls_simulate` (request shape identical — **no delta**) |
| `validateWorld(worldJson)` (optional) | `ls_validate_world` (accepts the bare world) |
| `memoryBytes()` (optional) | `exports.memory.buffer.byteLength` |

The playground's `WasmRuntime` only requires `version` + `simulate`; it size- and
range-checks requests before calling and JSON-parses the returned string into a
`TrajectoryInput`. The `simulate` request/response shapes match exactly.

**Delta:** the playground's `wasm.ts` does not currently call `derivative`,
`evalExpression`, or bundle round-trips — those glue helpers and C-ABI exports
are provided for completeness and are ready when the app opts in. Wiring the glue
into `apps/playground` (asset copy + `loader`) is a separate integration step and
is intentionally not done here (this crate does not edit `apps/playground/**`).

---

## Build

```bash
scripts/build-wasm.sh          # release build, stages web/lawsynth_wasm_bindings.wasm
PROFILE=debug scripts/build-wasm.sh
```

The script runs the exact documented steps:

```bash
rustup target add wasm32-unknown-unknown        # one-time, needs network
cargo build -p lawsynth-wasm-bindings --release --target wasm32-unknown-unknown
# artifact: target/wasm32-unknown-unknown/release/lawsynth_wasm_bindings.wasm
# staged to: crates/lawsynth-wasm-bindings/web/lawsynth_wasm_bindings.wasm
# (and into apps/playground/public/ if that directory exists)
```

The `.wasm` needs no import object — instantiate with `{}`.

---

## Status (honest)

- ✅ **Host-compiled & host-tested.** `cargo test -p lawsynth-wasm-bindings
  --offline` passes: the tests call the `extern "C"` exports through the real
  C-ABI (alloc a buffer, write request JSON, call the entry point, read the
  length-prefixed result, parse, assert). Coverage includes an analytic RK4
  check (`x' = -x` → `e⁻¹`), parameter folding (`e⁻²`), derivative evaluation,
  expression evaluation, the error path (`INVALID_WORLD`, `UNSUPPORTED`), the
  `MEMORY_LIMIT` guard, `version`, bare-world validation, and a bundle round-trip.
- ✅ `cargo clippy --all-targets -- -D warnings` clean; `cargo fmt` clean;
  `node --check web/lawsynth_wasm.mjs` clean.
- ⚠️ **The `.wasm` artifact was NOT produced in this environment.** Building it
  requires the `wasm32-unknown-unknown` target, whose std must be downloaded via
  `rustup target add` (network). This environment is offline with that target
  uninstalled, so `scripts/build-wasm.sh` is written and verified but not run
  here. Run it on a networked machine / CI to emit the deployable module.
- ⚠️ **Playground wiring is a separate step.** The glue matches `wasm.ts`'s
  `simulate`/`version` contract exactly, but this crate does not modify
  `apps/playground/**`.

Any `unsafe` is confined to the pointer ABI in [`src/ffi.rs`](src/ffi.rs) and the
test harness, each block annotated with a `// SAFETY:` invariant.
