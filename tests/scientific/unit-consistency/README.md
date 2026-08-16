# Unit-consistency boundary

The Rust world model has typed units, but the current public CSV discovery
command accepts only numeric headers and values. This case proves the CLI
produces a valid executable world while documenting that it cannot infer,
validate, or preserve physical units from a plain CSV alone.
