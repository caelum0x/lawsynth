# lawsynth-sklearn

scikit-learn-compatible estimators for [LawSynth](../lawsynth) — deterministic,
offline governing-equation discovery that drops into the sklearn ecosystem with
near-zero switching cost.

Three adapters wrap LawSynth's `Study` discovery loop in the scikit-learn
estimator contract (`fit` / `predict` / `score` / `get_params` / `set_params` /
`__sklearn_tags__` / `NotFittedError`):

| Estimator | Mixin | What it does |
|---|---|---|
| `LawSynthDynamics` | `BaseEstimator` | **Flagship.** Discover governing dynamics from a multivariate time-series; `predict` / `simulate` forecast trajectories, `score` returns trajectory R², `equations()` returns readable laws. |
| `LawSynthRegressor` | `RegressorMixin` | Strict static-fit framing for `Pipeline` / `GridSearchCV`: discovers the coupled world for `[predictors…, target]` and reconstructs the target trajectory. |
| `LawSynthTransformer` | `TransformerMixin` | Emit the polynomial / trigonometric feature library, with an optional deterministic correlation prune (least-correlated subset). |

## Install

```bash
pip install -e .            # requires the sibling `lawsynth` package
pip install -e .[sklearn]   # add scikit-learn + numpy for full interop
```

The estimators inherit real scikit-learn mixins **when sklearn is installed**,
and degrade to a standalone implementation of the **same** contract when it is
absent — the package stays importable and usable with plain Python lists in a
pure-standard-library, offline environment.

## Quickstart

```python
import math
from lawsynth_sklearn import LawSynthDynamics

t = [i * 0.05 for i in range(160)]
X = [[math.cos(ti), -math.sin(ti)] for ti in t]        # columns: [x, v]

dyn = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
print(dyn.equations())          # {'x': 'dx/dt = 0.9996·v', 'v': 'dv/dt = -0.9996·x'}
print(dyn.score(X, t))          # ~1.0  (trajectory R²)
traj = dyn.simulate(horizon=2.0, initial={"x": 1.0, "v": 0.0})
```

See [`examples/sklearn_quickstart.py`](examples/sklearn_quickstart.py) for the
Pipeline / clone / transformer / auto-parsimony walk-through.

## Auto-parsimony

Pass `parsimony="auto"` to `LawSynthDynamics` / `LawSynthRegressor` to select the
sparsity threshold automatically. It sweeps a deterministic grid of thresholds,
filters candidates to the `(complexity, loss)` Pareto front, and prices
complexity with

```
λ = Cov(complexity, loss) / Var(complexity)
```

(the same heuristic as gplearn's `parsimony_coefficient='auto'`), then selects
the model minimising `loss + |λ|·complexity`. The chosen threshold and λ are
exposed as `parsimony_coefficient_` and the scored sweep as
`parsimony_candidates_`.

## Determinism

LawSynth discovery is bit-exact and offline; every path here — the threshold
sweep, the correlation prune, the feature library — is deterministic, so
identical inputs reproduce identical estimators, equations, and forecasts.
