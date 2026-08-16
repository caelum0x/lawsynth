# Python SDK

The `lawsynth` package has a typed pure-Python facade and an optional compiled `_native` extension. Validation models such as `Dataset`, `DiscoveryConfig`, `Scenario`, and `TrajectoryData` import without the extension. Executing discovery, constructing a native `World`, loading bundles, and simulation require a built package containing `_native`; a missing extension raises `NativeError` at the execution boundary.

The public root exports `Dataset`, `DiscoveryConfig`, `LawSynthError`, `ValidationError`, `NativeError`, `discover`, `World`, `Scenario`, and `Trajectory`. Native classes are lazily resolved so data validation remains usable in source-only environments.
