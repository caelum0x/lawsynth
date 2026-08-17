# Usage: neural-prior

Evaluate a small dense neural network as a prior / feature transform over numeric
feature rows. The forward pass is deterministic, dependency-free, and strictly
bounded.

## Install

```bash
pip install -e plugins/neural-prior
```

Core inference uses only the standard-library `math` module. `numpy` and `torch`
are optional extras used only by external weight-export tooling:

```bash
pip install -e "plugins/neural-prior[numpy]"
```

## Model schema

```python
model = {
    "layers": [
        {"weights": [[...], ...], "bias": [...], "activation": "relu"},
        ...
    ]
}
```

- `weights` is a `rows x cols` matrix; `rows` = layer output width, `cols` =
  input width.
- `bias` length must equal the number of output rows.
- Consecutive layers must connect (each layer's input width equals the previous
  layer's output width).
- Activations: `identity`, `relu`, `tanh`, `sigmoid`.
- Every parameter must be finite; the total parameter count is capped by
  `max_parameters` (default 10,000,000).

Invalid shapes or parameters raise `ValueError` at construction.

## API

```python
from neural_prior.plugin import NeuralPrior

prior = NeuralPrior(model, max_parameters=10_000_000)

prior.predict([1.0, 2.0])                 # -> [float, ...] for one feature row
prior.invoke({"features": [[1.0, 2.0]]})  # -> {"predictions": [[...]], "row_count": 1}
```

The feature width must equal the first layer's input width, or a `ValueError`
is raised. Batches larger than 1,000,000 rows are rejected.

## Example

```python
model = {
    "layers": [
        {"weights": [[0.5, -0.5], [1.0, 0.0]], "bias": [0.0, 0.1], "activation": "relu"},
        {"weights": [[1.0, 1.0]], "bias": [-0.25], "activation": "identity"},
    ]
}
NeuralPrior(model).invoke({"features": [[1.0, 2.0], [0.0, -1.0]]})
```

## Numerical notes

- The `sigmoid` input is clamped to `[-700, 700]` to avoid overflow.
- Computation is pure Python `float`; results are deterministic across runs.

## As a discovery prior

Use the predictions as engineered features alongside raw state columns when
building a `lawsynth.dataset.Dataset`, letting discovery search over a learned
nonlinear feature map without importing a heavyweight ML runtime into the data
path.
