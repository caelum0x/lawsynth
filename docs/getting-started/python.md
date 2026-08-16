# Python

The Python package has a small pure-Python validation/configuration layer and
a native extension backed by the Rust workspace. Build it in place from
`python/lawsynth`:

```sh
python -m pip install maturin
maturin develop
python -m pytest -q tests
```

The configuration and data classes can be imported without a native build.
Discovery, native World construction, bundle IO, and simulation require
`lawsynth._native`; calling them without the extension raises a clear import
error instead of falling back to an unrelated implementation.

```python
from lawsynth import Dataset, DiscoveryConfig, discover

dataset = Dataset.from_columns([0.0, 0.1, 0.2], {"x": [1.0, 0.98, 0.96]})
world = discover(dataset, ["x"], DiscoveryConfig())
```

Use only finite numeric observations and valid identifiers. The Python layer
does not add support for stochastic, regime, causal, or delayed Worlds.
