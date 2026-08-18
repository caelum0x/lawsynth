//! C-ABI (wasm32) bindings over `lawsynth-wasm` for the browser playground.
//!
//! This crate exposes a small, hand-rolled `#[no_mangle] extern "C"` surface — a
//! `Vec`-backed linear-memory allocator plus string-in / string-out entry points —
//! so the playground can drive the LawSynth simulation core from JavaScript
//! WITHOUT `wasm-bindgen` and WITHOUT any external crate. That keeps the build
//! fully offline and the `.wasm` lean. It compiles as a `cdylib` (the deployable
//! `.wasm`) and as an `rlib`, letting the host test binary exercise the exact
//! exported functions across the same C-ABI a JavaScript caller would use.
//!
//! Everything here is deterministic: fixed-step RK4, pure expression evaluation,
//! and byte-exact bundle codecs. No wall-clock, no RNG, no filesystem, no network.
//!
//! See [`ffi`] for the full memory & packing protocol, and `README.md` for the
//! per-export request/response contract and the (network-gated) build step.

mod api;
mod convert;
mod ffi;
mod json;

// Re-export the C-ABI so the exported symbols are reachable from the `rlib`
// (host tests) and documented as the crate's public surface.
pub use ffi::{
    ls_alloc, ls_bundle_decode, ls_bundle_encode, ls_derivative, ls_eval_expression, ls_free,
    ls_last_error, ls_last_error_len, ls_result_len, ls_simulate, ls_validate_world, ls_version,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::{Json, parse};

    /// Result header: 4-byte little-endian length + 1-byte status.
    const HEADER: usize = 5;

    /// Drive an entry point exactly as JavaScript would: allocate an input
    /// buffer, copy bytes in, call the export, then read and free the packed
    /// result. Returns `(status, payload)`.
    fn call(
        entry: unsafe extern "C" fn(*const u8, usize) -> *mut u8,
        input: &[u8],
    ) -> (u8, Vec<u8>) {
        let ptr = ls_alloc(input.len());
        // SAFETY: `ptr` addresses `input.len()` writable bytes just allocated.
        unsafe { std::ptr::copy_nonoverlapping(input.as_ptr(), ptr, input.len()) };
        // SAFETY: `ptr`/`len` name the buffer we just filled.
        let result = unsafe { entry(ptr.cast_const(), input.len()) };
        // SAFETY: input buffer allocated with this exact length.
        unsafe { ls_free(ptr, input.len()) };
        read_result(result)
    }

    /// Read `(status, payload)` from a result buffer and free it.
    fn read_result(result: *mut u8) -> (u8, Vec<u8>) {
        // SAFETY: `result` is a packed buffer returned by an entry point.
        let n = unsafe { ls_result_len(result.cast_const()) };
        // SAFETY: the buffer holds HEADER + n valid bytes.
        let slice = unsafe { std::slice::from_raw_parts(result, HEADER + n) };
        let status = slice[4];
        let payload = slice[HEADER..HEADER + n].to_vec();
        // SAFETY: result buffers are freed with their full HEADER + n length.
        unsafe { ls_free(result, HEADER + n) };
        (status, payload)
    }

    fn payload_json(payload: &[u8]) -> Json {
        parse(std::str::from_utf8(payload).expect("utf8")).expect("json")
    }

    fn get_f64(value: &Json, path: &[&str]) -> f64 {
        let mut node = value;
        for key in path {
            node = node.get(key).expect("key");
        }
        node.as_f64().expect("number")
    }

    const DECAY_WORLD: &str = r#"{
        "world": {
            "formatVersion": "0.1.0",
            "id": "decay",
            "time": { "kind": "continuous", "symbol": "t" },
            "variables": [{ "id": "x", "role": "state" }],
            "laws": [{
                "kind": "continuous",
                "target": "x",
                "expression": { "kind": "unary", "operator": "neg", "operand": { "kind": "symbol", "id": "x" } }
            }]
        },
        "initial": { "x": 1 },
        "start": 0, "end": 1, "step": 0.01
    }"#;

    #[test]
    fn simulate_exponential_decay_matches_analytic_solution() {
        let (status, payload) = call(ls_simulate, DECAY_WORLD.as_bytes());
        assert_eq!(status, 0, "expected OK status");
        let value = payload_json(&payload);

        // Shape check: TrajectoryInput { variables, times, values }.
        let variables = value.get("variables").and_then(Json::as_array).expect("variables");
        assert_eq!(variables.len(), 1);
        assert_eq!(variables[0].as_str(), Some("x"));

        let times = value.get("times").and_then(Json::as_array).expect("times");
        let values = value.get("values").and_then(Json::as_array).expect("values");
        assert_eq!(times.len(), values.len());
        assert_eq!(times[0].as_f64(), Some(0.0));
        assert_eq!(values[0].as_array().unwrap()[0].as_f64(), Some(1.0));

        // RK4 of x' = -x over [0,1] must approach e^-1 tightly.
        let last = values.last().unwrap().as_array().unwrap()[0].as_f64().unwrap();
        let expected = (-1.0_f64).exp();
        assert!((last - expected).abs() < 1e-6, "got {last}, expected {expected}");
    }

    #[test]
    fn simulate_resolves_parameters_to_constants() {
        // x' = -(k * x), k = 2  =>  x(1) = e^-2.
        let request = r#"{
            "world": {
                "formatVersion": "0.1.0", "id": "pdecay",
                "time": { "kind": "continuous", "symbol": "t" },
                "variables": [{ "id": "x", "role": "state" }],
                "parameters": [{ "id": "k", "value": 2 }],
                "laws": [{ "kind": "continuous", "target": "x", "expression": {
                    "kind": "unary", "operator": "neg", "operand": {
                        "kind": "binary", "operator": "mul",
                        "left": { "kind": "symbol", "id": "k" },
                        "right": { "kind": "symbol", "id": "x" } } } }]
            },
            "initial": { "x": 1 },
            "parameters": { "k": 2 },
            "start": 0, "end": 1, "step": 0.01
        }"#;
        let (status, payload) = call(ls_simulate, request.as_bytes());
        assert_eq!(status, 0);
        let value = payload_json(&payload);
        let values = value.get("values").and_then(Json::as_array).unwrap();
        let last = values.last().unwrap().as_array().unwrap()[0].as_f64().unwrap();
        assert!((last - (-2.0_f64).exp()).abs() < 1e-6, "got {last}");
    }

    #[test]
    fn derivative_evaluates_field_at_a_point() {
        let request = r#"{
            "world": {
                "formatVersion": "0.1.0", "id": "decay",
                "time": { "kind": "continuous", "symbol": "t" },
                "variables": [{ "id": "x", "role": "state" }],
                "laws": [{ "kind": "continuous", "target": "x", "expression": {
                    "kind": "unary", "operator": "neg", "operand": { "kind": "symbol", "id": "x" } } }]
            },
            "state": { "x": 2 },
            "t": 0
        }"#;
        let (status, payload) = call(ls_derivative, request.as_bytes());
        assert_eq!(status, 0);
        let value = payload_json(&payload);
        let derivative = value.get("derivative").and_then(Json::as_array).unwrap();
        assert_eq!(derivative.len(), 1);
        assert_eq!(derivative[0].as_f64(), Some(-2.0));
    }

    #[test]
    fn eval_expression_parses_and_evaluates() {
        let request = r#"{ "expression": "sin(0) + cos(0)", "scope": {} }"#;
        let (status, payload) = call(ls_eval_expression, request.as_bytes());
        assert_eq!(status, 0);
        assert_eq!(get_f64(&payload_json(&payload), &["value"]), 1.0);

        let scoped = r#"{ "expression": "2 * y + 1", "scope": { "y": 3 } }"#;
        let (status, payload) = call(ls_eval_expression, scoped.as_bytes());
        assert_eq!(status, 0);
        assert_eq!(get_f64(&payload_json(&payload), &["value"]), 7.0);
    }

    #[test]
    fn invalid_world_reports_error_code() {
        // No continuous law for the declared state variable.
        let request = r#"{
            "world": { "formatVersion": "0.1.0", "id": "broken",
                "time": { "kind": "continuous", "symbol": "t" },
                "variables": [{ "id": "x", "role": "state" }], "laws": [] },
            "initial": { "x": 1 }, "start": 0, "end": 1, "step": 0.1
        }"#;
        let (status, payload) = call(ls_simulate, request.as_bytes());
        assert_eq!(status, 1, "expected ERROR status");
        let value = payload_json(&payload);
        assert_eq!(value.get("code").and_then(Json::as_str), Some("INVALID_WORLD"));
        // The out-of-band error channel must mirror the envelope code.
        assert_eq!(last_error_code(), "INVALID_WORLD");
    }

    #[test]
    fn unsupported_expression_kind_reports_unsupported() {
        let request = r#"{
            "world": { "formatVersion": "0.1.0", "id": "u",
                "time": { "kind": "continuous", "symbol": "t" },
                "variables": [{ "id": "x", "role": "state" }],
                "laws": [{ "kind": "continuous", "target": "x", "expression": {
                    "kind": "logical", "operator": "and", "operands": [] } }] },
            "initial": { "x": 1 }, "start": 0, "end": 1, "step": 0.1
        }"#;
        let (status, payload) = call(ls_simulate, request.as_bytes());
        assert_eq!(status, 1);
        assert_eq!(payload_json(&payload).get("code").and_then(Json::as_str), Some("UNSUPPORTED"));
    }

    #[test]
    fn oversized_request_reports_memory_limit_without_reading() {
        // Pass a tiny, valid pointer but declare a length beyond the request
        // budget: the guard must fire BEFORE any dereference of the buffer.
        let ptr = ls_alloc(1);
        let huge = api::MAX_REQUEST_BYTES + 1;
        // SAFETY: `ls_simulate` checks the size guard before touching `ptr`, so
        // the oversized `len` is never used to read memory.
        let result = unsafe { ls_simulate(ptr.cast_const(), huge) };
        // SAFETY: input allocated with length 1.
        unsafe { ls_free(ptr, 1) };
        let (status, payload) = read_result(result);
        assert_eq!(status, 1);
        assert_eq!(payload_json(&payload).get("code").and_then(Json::as_str), Some("MEMORY_LIMIT"));
    }

    #[test]
    fn version_export_returns_crate_version() {
        let result = ls_version();
        let (status, payload) = read_result(result);
        assert_eq!(status, 0);
        assert_eq!(std::str::from_utf8(&payload).unwrap(), env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn validate_world_reports_dimension() {
        let (status, payload) = call(ls_validate_world, DECAY_WORLD.as_bytes());
        assert_eq!(status, 0);
        let value = payload_json(&payload);
        assert_eq!(value.get("ok"), Some(&Json::Bool(true)));
        assert_eq!(get_f64(&value, &["dimension"]), 1.0);
    }

    #[test]
    fn validate_world_accepts_a_bare_world_document() {
        // Matches the playground's `validateWorld(worldJson)` signature: no
        // envelope, no initial state required.
        let bare_world = r#"{
            "formatVersion": "0.1.0", "id": "decay",
            "time": { "kind": "continuous", "symbol": "t" },
            "variables": [{ "id": "x", "role": "state" }],
            "laws": [{ "kind": "continuous", "target": "x", "expression": {
                "kind": "unary", "operator": "neg", "operand": { "kind": "symbol", "id": "x" } } }]
        }"#;
        let (status, payload) = call(ls_validate_world, bare_world.as_bytes());
        assert_eq!(status, 0);
        assert_eq!(get_f64(&payload_json(&payload), &["dimension"]), 1.0);
    }

    #[test]
    fn bundle_round_trips_through_the_abi() {
        // Encode (text in -> binary out), then decode (binary in -> text out).
        let (status, bundle_bytes) = call(ls_bundle_encode, DECAY_WORLD.as_bytes());
        assert_eq!(status, 0);
        assert!(bundle_bytes.starts_with(b"LSWASM"), "expected bundle magic");

        let (status, payload) = call(ls_bundle_decode, &bundle_bytes);
        assert_eq!(status, 0);
        let value = payload_json(&payload);
        let variables = value.get("variables").and_then(Json::as_array).unwrap();
        assert_eq!(variables[0].as_str(), Some("x"));
        assert_eq!(get_f64(&value, &["initial", "x"]), 1.0);
    }

    fn last_error_code() -> String {
        let len = ls_last_error_len();
        let ptr = ls_last_error();
        // SAFETY: `ptr`/`len` describe the current thread-local error bytes,
        // which stay valid until the next binding call.
        let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
        String::from_utf8(bytes.to_vec()).unwrap()
    }
}
