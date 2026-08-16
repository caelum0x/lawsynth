# Limits and rejection behavior

`BundleConfig::default()` expresses the intended admission policy: at most 64 entries, largest entry at most 64 MiB, and total entry content at most 256 MiB. `BundleConfig::accepts` is a pure predicate for callers that preflight untrusted inputs; the current `read_world` and `read_discrete_world` APIs do not take a config and therefore do not enforce those three aggregate limits themselves.

Codec-enforced limits are u16 ZIP entry names and binary strings, u16 ZIP entry count, u32 ZIP sizes and offsets, u32 encoded component counts, finite parameter and constant values, and expression nesting strictly below 128 (the root is depth 0; a node reached with depth 128 is rejected). All byte slicing and offset arithmetic is checked. Violations return `BundleError` rather than partial artifacts.

The format supports stored ZIP entries only. It intentionally has no page decoder, compression codec, encrypted ZIP entry, ZIP64, streaming-reader, or recovery mode.
