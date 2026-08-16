# LawSynth plugin host

This host provides the policy boundary around plugin API contracts. Plugins are
disabled by default. Enabling them is not sufficient: requested capabilities
must be part of an administrator-provided allow list, and trusted native plugins
require a separate opt-in.

The host can discover manifests, register validated plugins, spawn process
plugins without shell interpolation, meter requests/output/time, and transport
bounded protocol frames over child pipes. WASI bytes are validated before being
handed to an external runtime; this crate does not silently execute a component
with an undeclared runtime or permission model.
