//! Link a Python extension the same way maturin does on macOS.
//!
//! Extension modules resolve CPython API symbols from the interpreter that
//! imports them, rather than linking a particular `libpython` at build time.
//! PyO3 provides the target-aware flags; it is a no-op on platforms where no
//! special flags are required.
fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
