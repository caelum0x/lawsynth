"""scikit-learn integration — skipped cleanly when sklearn is absent."""

from __future__ import annotations

import pytest

sklearn = pytest.importorskip("sklearn")

from sklearn import clone  # noqa: E402
from sklearn.base import is_regressor  # noqa: E402
from sklearn.pipeline import Pipeline  # noqa: E402

from lawsynth_sklearn import (  # noqa: E402
    LawSynthDynamics,
    LawSynthRegressor,
    LawSynthTransformer,
)


def test_clone_dynamics(oscillator):
    est = LawSynthDynamics(polynomial_degree=1, threshold=0.05)
    cloned = clone(est)
    assert cloned.get_params() == est.get_params()
    assert cloned is not est


def test_clone_after_fit_forgets_state(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    cloned = clone(est)
    assert not hasattr(cloned, "world_")


def test_regressor_is_recognized():
    assert is_regressor(LawSynthRegressor())


def test_regressor_in_pipeline(oscillator):
    X, t = oscillator
    x_col = [[row[0]] for row in X]
    v_target = [row[1] for row in X]
    dt = t[1] - t[0]
    pipe = Pipeline([
        ("features", LawSynthTransformer(degree=1)),
        ("law", LawSynthRegressor(polynomial_degree=1, threshold=0.05, dt=dt, target_name="v")),
    ])
    pipe.fit(x_col, v_target)
    assert pipe.score(x_col, v_target) > 0.99
    assert pipe.named_steps["law"].equation().startswith("dv/dt")


def test_transformer_then_linear_model(oscillator):
    from sklearn.linear_model import LinearRegression

    X, _ = oscillator
    target = [row[1] for row in X]
    pipe = Pipeline([
        ("lib", LawSynthTransformer(degree=2, include_bias=False)),
        ("lr", LinearRegression()),
    ])
    pipe.fit(X, target)
    assert pipe.score(X, target) > 0.99


def test_transformer_feature_names_out_is_array(oscillator):
    import numpy as np

    X, _ = oscillator
    tr = LawSynthTransformer(degree=2, include_bias=True).fit(X)
    names = tr.get_feature_names_out()
    assert isinstance(names, np.ndarray)


def test_grid_search_over_regressor(oscillator):
    from sklearn.model_selection import GridSearchCV

    X, t = oscillator
    x_col = [[row[0]] for row in X]
    v_target = [row[1] for row in X]
    dt = t[1] - t[0]
    grid = GridSearchCV(
        LawSynthRegressor(dt=dt, target_name="v"),
        {"polynomial_degree": [1, 2], "threshold": [0.02, 0.05]},
        cv=2,
    )
    grid.fit(x_col, v_target)
    assert hasattr(grid, "best_params_")
