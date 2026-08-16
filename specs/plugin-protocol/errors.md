# Errors

PluginError distinguishes invalid manifests/capabilities/data/limits, invalid
lifecycle transitions, protocol violations, resource-limit violations, and
unsupported features. Error text is diagnostic; callers MUST branch on the
variant in Rust rather than parsing display strings.

Validation errors are returned before a value crosses the plugin boundary. There
is no numeric error registry, retry classification, remote error envelope, or
guarantee that plugin-defined algorithm diagnostics are machine-readable.
