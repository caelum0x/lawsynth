# LawSynth Python SDK

The Python package supplies validated data models around the compiled
`lawsynth._native` executable-world extension. It keeps dataset and discovery
configuration validation available before native loading, while World creation,
bundle I/O, discovery, and simulation remain implemented by the Rust engine.

```python
from lawsynth.dataset import Dataset
from lawsynth.discover import discover

dataset = Dataset.from_columns([0.0, 1.0, 2.0], {"x": [1.0, 2.0, 4.0]})
world = discover(dataset, ("x",))
```

Build with `maturin develop` from this directory, or install a wheel built by
the repository release process. The package requires Python 3.11 or newer.
