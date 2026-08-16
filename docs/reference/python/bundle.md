# Python bundles

`lawsynth.bundle.save(world, path)` delegates to the native world's deterministic bundle writer. `lawsynth.bundle.load(path)` calls `World.load` after the native extension is available. Both operate on continuous native worlds.

The format is a validated `.lsworld` archive, not a generic ZIP interchange API. Python does not expose bundle migration, archive extraction, signature storage, discrete-world loading, or partial reads.
