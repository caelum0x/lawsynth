//! The hand-rolled C-ABI surface consumed by the browser glue.
//!
//! # Memory & packing protocol
//!
//! JavaScript drives the module entirely through raw pointers into the WASM
//! linear memory:
//!
//! 1. `ls_alloc(len)` returns a pointer to `len` writable, zeroed bytes.
//! 2. JS copies a UTF-8 (or, for `ls_bundle_decode`, binary) request into it.
//! 3. JS calls an entry point `entry(ptr, len)` which returns a pointer to a
//!    **result buffer** laid out as:
//!
//!    ```text
//!    byte 0..4   u32 little-endian  N   (payload length in bytes)
//!    byte 4      u8                 status  (0 = OK, 1 = ERROR)
//!    byte 5..5+N payload bytes
//!    ```
//!
//!    On `OK` the payload is the operation's result (bare `TrajectoryInput`
//!    JSON, etc.); on `ERROR` the payload is `{"code","message"}` JSON.
//! 4. JS reads `N` + `status`, copies the payload out, then calls
//!    `ls_free(resultPtr, 5 + N)`.
//! 5. JS frees its own input buffer with `ls_free(inputPtr, inputLen)`.
//!
//! A stable machine-readable error code for the most recent call is also exposed
//! out-of-band via `ls_last_error()` / `ls_last_error_len()`.

use std::cell::RefCell;
use std::panic::catch_unwind;

use lawsynth_wasm::{WasmError, error_code};

use crate::api;
use crate::json::Json;

/// Fixed header size: 4-byte length prefix + 1-byte status.
const HEADER: usize = 5;

const STATUS_OK: u8 = 0;
const STATUS_ERR: u8 = 1;

thread_local! {
    /// NUL-free machine-readable code of the most recent operation. Its pointer
    /// (from `ls_last_error`) stays valid until the next binding call mutates it.
    static LAST_ERROR: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn set_last_error(code: &str) {
    LAST_ERROR.with(|cell| {
        let mut slot = cell.borrow_mut();
        slot.clear();
        slot.extend_from_slice(code.as_bytes());
    });
}

fn clear_last_error() {
    LAST_ERROR.with(|cell| cell.borrow_mut().clear());
}

/// Allocate `len` zeroed, writable bytes and hand ownership to the caller.
///
/// The returned pointer must be released with [`ls_free`] using the **same**
/// `len`. Allocating with `vec![0u8; len]` guarantees `capacity == len`, which
/// keeps the later `Vec::from_raw_parts` reconstruction sound.
#[unsafe(no_mangle)]
pub extern "C" fn ls_alloc(len: usize) -> *mut u8 {
    let mut buffer = vec![0u8; len];
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// Free a buffer previously produced by [`ls_alloc`] or any entry point.
///
/// # Safety
///
/// `ptr` must have been returned by this module and paired with the exact `len`
/// originally allocated (for result buffers, `len == 5 + N`). Calling this twice
/// on the same pointer, or with a mismatched length, is undefined behavior.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_free(ptr: *mut u8, len: usize) {
    if ptr.is_null() {
        return;
    }
    // SAFETY: by contract `ptr`/`len` name a live allocation from this module
    // whose capacity equals `len`; reconstructing the `Vec` reclaims it exactly
    // once, and the caller promises not to use `ptr` afterwards.
    unsafe {
        drop(Vec::from_raw_parts(ptr, len, len));
    }
}

/// Read the payload length `N` from a result buffer's 4-byte prefix.
///
/// # Safety
///
/// `ptr` must point to a result buffer previously returned by an entry point
/// (at least 4 readable bytes).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_result_len(ptr: *const u8) -> usize {
    if ptr.is_null() {
        return 0;
    }
    // SAFETY: caller guarantees `ptr` addresses a result buffer with a valid
    // 4-byte little-endian length prefix.
    let header = unsafe { std::slice::from_raw_parts(ptr, 4) };
    u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize
}

/// Pointer to the most-recent error code bytes (empty when the last call
/// succeeded). Valid until the next binding call. Length via [`ls_last_error_len`].
#[unsafe(no_mangle)]
pub extern "C" fn ls_last_error() -> *const u8 {
    LAST_ERROR.with(|cell| cell.borrow().as_ptr())
}

/// Byte length of the most-recent error code (0 when the last call succeeded).
#[unsafe(no_mangle)]
pub extern "C" fn ls_last_error_len() -> usize {
    LAST_ERROR.with(|cell| cell.borrow().len())
}

/// Return the crate version as a packed OK result buffer.
#[unsafe(no_mangle)]
pub extern "C" fn ls_version() -> *mut u8 {
    clear_last_error();
    pack(STATUS_OK, env!("CARGO_PKG_VERSION").as_bytes())
}

/// Simulate a world (`ls_simulate`), returning `TrajectoryInput` JSON.
///
/// # Safety
/// See the module-level protocol: `ptr`/`len` must name a UTF-8 request buffer
/// from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_simulate(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_text`, which upholds the ptr/len contract.
    unsafe { run_text(ptr, len, api::simulate) }
}

/// Validate a world without simulating (`ls_validate_world`).
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_validate_world(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_text`, which upholds the ptr/len contract.
    unsafe { run_text(ptr, len, api::validate_world) }
}

/// Evaluate the derivative field at a point (`ls_derivative`).
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_derivative(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_text`, which upholds the ptr/len contract.
    unsafe { run_text(ptr, len, api::derivative) }
}

/// Parse and evaluate one scalar expression (`ls_eval_expression`).
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_eval_expression(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_text`, which upholds the ptr/len contract.
    unsafe { run_text(ptr, len, api::eval_expression) }
}

/// Encode a world into a `.lsworld` bundle (`ls_bundle_encode`). The OK payload
/// is binary bundle bytes, not UTF-8.
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_bundle_encode(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_bytes_out`, which upholds the ptr/len contract.
    unsafe { run_bytes_out(ptr, len, api::bundle_encode) }
}

/// Decode a `.lsworld` bundle into JSON (`ls_bundle_decode`). The input buffer
/// is raw bundle bytes.
///
/// # Safety
/// `ptr`/`len` must name a binary request buffer from [`ls_alloc`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ls_bundle_decode(ptr: *const u8, len: usize) -> *mut u8 {
    // SAFETY: forwarded to `run_bytes_in`, which upholds the ptr/len contract.
    unsafe { run_bytes_in(ptr, len, api::bundle_decode) }
}

/// Read the input slice, guarding size before any dereference.
///
/// # Safety
/// `ptr`/`len` must name a readable buffer from [`ls_alloc`] (unless `len == 0`).
unsafe fn input_slice<'a>(ptr: *const u8, len: usize) -> Result<&'a [u8], WasmError> {
    api::check_request_size(len)?;
    if len == 0 {
        return Ok(&[]);
    }
    if ptr.is_null() {
        return Err(WasmError::InvalidWorld("null input pointer".into()));
    }
    // SAFETY: size is within bounds and `ptr` is a non-null buffer of `len`
    // initialized bytes per the caller contract.
    Ok(unsafe { std::slice::from_raw_parts(ptr, len) })
}

/// Runner for text-in / text-out operations.
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
unsafe fn run_text(
    ptr: *const u8,
    len: usize,
    op: fn(&str) -> Result<String, WasmError>,
) -> *mut u8 {
    // SAFETY: delegated to `input_slice`, which enforces the buffer contract.
    let bytes = match unsafe { input_slice(ptr, len) } {
        Ok(bytes) => bytes,
        Err(error) => return error_result(&error),
    };
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return error_result(&WasmError::InvalidWorld("request is not valid UTF-8".into()));
        }
    };
    match catch_unwind(|| op(text)) {
        Ok(Ok(payload)) => {
            clear_last_error();
            pack(STATUS_OK, payload.as_bytes())
        }
        Ok(Err(error)) => error_result(&error),
        Err(_) => {
            error_result(&WasmError::Simulation("internal panic while handling request".into()))
        }
    }
}

/// Runner for text-in / binary-out operations (bundle encode).
///
/// # Safety
/// `ptr`/`len` must name a UTF-8 request buffer from [`ls_alloc`].
unsafe fn run_bytes_out(
    ptr: *const u8,
    len: usize,
    op: fn(&str) -> Result<Vec<u8>, WasmError>,
) -> *mut u8 {
    // SAFETY: delegated to `input_slice`, which enforces the buffer contract.
    let bytes = match unsafe { input_slice(ptr, len) } {
        Ok(bytes) => bytes,
        Err(error) => return error_result(&error),
    };
    let text = match std::str::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => {
            return error_result(&WasmError::InvalidWorld("request is not valid UTF-8".into()));
        }
    };
    match catch_unwind(|| op(text)) {
        Ok(Ok(payload)) => {
            clear_last_error();
            pack(STATUS_OK, &payload)
        }
        Ok(Err(error)) => error_result(&error),
        Err(_) => {
            error_result(&WasmError::Simulation("internal panic while handling request".into()))
        }
    }
}

/// Runner for binary-in / text-out operations (bundle decode).
///
/// # Safety
/// `ptr`/`len` must name a binary request buffer from [`ls_alloc`].
unsafe fn run_bytes_in(
    ptr: *const u8,
    len: usize,
    op: fn(&[u8]) -> Result<String, WasmError>,
) -> *mut u8 {
    // SAFETY: delegated to `input_slice`, which enforces the buffer contract.
    let bytes = match unsafe { input_slice(ptr, len) } {
        Ok(bytes) => bytes,
        Err(error) => return error_result(&error),
    };
    match catch_unwind(|| op(bytes)) {
        Ok(Ok(payload)) => {
            clear_last_error();
            pack(STATUS_OK, payload.as_bytes())
        }
        Ok(Err(error)) => error_result(&error),
        Err(_) => {
            error_result(&WasmError::Simulation("internal panic while handling request".into()))
        }
    }
}

/// Record `error` out-of-band and pack an ERROR result envelope.
fn error_result(error: &WasmError) -> *mut u8 {
    let code = error_code(error);
    set_last_error(code);
    let envelope = Json::Obj(vec![
        ("code".to_string(), Json::Str(code.to_string())),
        ("message".to_string(), Json::Str(error.to_string())),
    ]);
    pack(STATUS_ERR, envelope.to_json_string().as_bytes())
}

/// Build a `[u32 len][u8 status][payload]` result buffer and hand it to JS.
fn pack(status: u8, payload: &[u8]) -> *mut u8 {
    let n = payload.len();
    // Sizing the Vec exactly keeps `capacity == len`, so the eventual
    // `ls_free(ptr, HEADER + n)` reconstruction is sound.
    let mut buffer = Vec::with_capacity(HEADER + n);
    buffer.extend_from_slice(&(n as u32).to_le_bytes());
    buffer.push(status);
    buffer.extend_from_slice(payload);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}
