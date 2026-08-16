//! Validated, portable LawSynth model data and deterministic simulation primitives.
//!
//! This is intentionally free of browser bindings and a WebAssembly runtime. It can be
//! compiled by a WASM target, while a host chooses `wasm-bindgen`/component bindings.
//! It does not claim browser I/O, JavaScript callbacks, or dynamic plugin execution.
mod bundle;
mod config;
mod error;
mod errors;
mod events;
mod expression;
mod memory;
mod simulate;
mod trajectory;
mod world;
pub use bundle::Bundle;
pub use config::WasmConfig;
pub use error::WasmError;
pub use errors::code as error_code;
pub use events::{Event, EventDirection, EventOccurrence};
pub use expression::{BinaryOp, Expression, Function};
pub use memory::MemoryBudget;
pub use simulate::simulate_rk4;
pub use trajectory::Trajectory;
pub use world::World;
