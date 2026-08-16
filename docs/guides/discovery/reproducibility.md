# Reproducible discovery

Capture the input dataset hash, selected column names, time unit, preprocessing source revision, full CLI command or `DiscoveryConfig`, LawSynth version, platform, and output bundle hash. These records are the minimum needed to reproduce a discovered model.

Use stable input ordering and keep `--state` order explicit. Re-run the same command in a clean directory and compare the output bundle and reported diagnostics. If a dependency or engine version changes, treat the result as a new run until equivalence has been demonstrated.

Bundle encoding is deterministic for the same engine inputs, but reproducibility does not eliminate scientific uncertainty. Document validation data and analyst decisions separately from deterministic build metadata.
