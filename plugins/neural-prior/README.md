# neural-prior

A LawSynth `algorithm` plugin that evaluates a small dense (fully connected)
neural network as a **prior / feature transform** over numeric feature rows. It
is a deterministic, dependency-free forward pass with strict shape and parameter
bounds — safe to run inside a resource-limited plugin host.

Priors like this let discovery incorporate a learned nonlinear feature map (for
example, a pre-fit encoder) without pulling a heavyweight ML runtime into the
data path.

## Model schema

A model is a plain mapping of layers. Each layer carries a weight matrix, a bias
vector, and an activation name:

```python
model = {
    "layers": [
        {"weights": [[0.5, -0.5], [1.0, 0.0]], "bias": [0.0, 0.1], "activation": "relu"},
        {"weights": [[1.0, 1.0]],              "bias": [0.0],       "activation": "identity"},
    ]
}
```

Supported activations: `identity`, `relu`, `tanh`, `sigmoid`.

Validation is strict: layer widths must connect, every parameter must be finite,
and the total parameter count is capped (`max_parameters`, default 10M). The
sigmoid input is clamped to avoid overflow.

## Contract

```python
from neural_prior.plugin import NeuralPrior

prior = NeuralPrior(model)
result = prior.invoke({"features": [[1.0, 2.0], [0.0, -1.0]]})
# result == {"predictions": [[...], [...]], "row_count": 2}

# Single row:
prior.predict([1.0, 2.0])  # -> [float, ...]
```

The feature width must match the first layer's input width, or a `ValueError`
is raised.

## Optional accelerated backends

Core inference uses only the standard library `math` module. `numpy` and `torch`
are declared as **optional** extras and are only needed by external tooling that
exports trained weights into the plain-list schema above — importing this plugin
never imports them, and the plugin degrades to pure Python if they are absent.

## Install

```bash
pip install -e plugins/neural-prior
# optional, only for weight-export tooling:
pip install -e "plugins/neural-prior[numpy]"
```

See [docs/usage.md](docs/usage.md) and [examples/basic.py](examples/basic.py).
