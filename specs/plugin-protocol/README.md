# Plugin protocol

lawsynth-plugin-api is a dependency-free contract for isolated plugins. It
defines validated manifests, capability declarations, resource limits, lifecycle
states, typed data and simulation payloads, and a versioned frame format. It
does not provide a plugin host, loader, sandbox, registry, or service endpoint.

Plugins may be WASI components, isolated processes, or explicit trusted-native
extensions, but the selected kind is metadata only until a host enforces it.
Every host MUST validate input and output before it enters a World or run.
