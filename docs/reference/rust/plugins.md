# Plugin boundary

The production workspace does not compile a plugin runtime. There is no dynamic loading, native ABI, WASM host, marketplace client, or untrusted-code sandbox in the supported executable path. Extending a model requires constructing a supported expression/world with the Rust API and rebuilding the application.
