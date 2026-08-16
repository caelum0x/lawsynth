# lawsynth-wasm

This crate is the portable, deterministic LawSynth surface for validated continuous worlds, scalar expressions, RK4 trajectories, crossing events, and a compact versioned bundle format. It is dependency-free and can be compiled to a WebAssembly target.

It intentionally does **not** provide JavaScript bindings, browser storage, a browser event loop, WASI networking, plugin execution, or arbitrary code loading. Those are host concerns; an embedding can layer `wasm-bindgen` or component bindings over these validated Rust APIs without misrepresenting this crate as a browser runtime.

Expressions accept finite scalar constants, variables, `+ - * / ^`, parentheses, and `sin`, `cos`, `exp`, `log`, `sqrt`, and `abs`. Simulation is fixed-step classical RK4 and returns an error for invalid domains or non-finite state.
