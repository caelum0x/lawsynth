# Manifest

The parser accepts one key = value per line, optional # comments, and only these
keys: id, version, kind, entrypoint, capabilities, max_cpu_millis,
max_memory_bytes, max_output_bytes, and max_requests. Unknown or duplicate keys
reject. This is not TOML despite the familiar syntax.

id is 1--96 lowercase ASCII letters/digits with internal hyphens only. version
is exactly unsigned major.minor.patch. kind is wasi, process, or trusted-native;
entrypoint cannot be empty, contain NUL, or contain .. . Trusted-native manifests
MUST declare process.execute.
