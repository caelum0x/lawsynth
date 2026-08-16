# Reproducibility

Preserve the exact input table, time and state-column selection, preprocessing record, discovery configuration, package version, command line, random seed where used, and the generated `.lsworld` bundle. A dataset fingerprint and canonical bundle representation are available in the core/data and bundle layers; retain both checksums with published outputs.

To rerun an analysis, create a new directory, copy immutable inputs, execute the recorded command, inspect the produced bundle, and compare its contents and simulation output against the recorded result. For floating-point trajectories, compare with a declared tolerance and identical solver start, end, and step; different hardware or compiler math implementations can change last-bit results.

The CLI is synchronous. It does not provide a run scheduler or resumable optimization checkpoint. External workflow tools must preserve failed runs and logs as well as successful artifacts.
