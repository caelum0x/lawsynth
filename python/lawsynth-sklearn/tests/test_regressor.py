"""LawSynthRegressor: static-fit framing over LawSynth discovery."""

from __future__ import annotations

import pytest

from lawsynth_sklearn import LawSynthRegressor


def _split(oscillator):
    """Predictor x, target v of the oscillator (v' = -x)."""
    X, t = oscillator
    x_col = [[row[0]] for row in X]
    v_target = [row[1] for row in X]
    return x_col, v_target, t[1] - t[0]


def test_fit_predict_score(oscillator):
    x_col, v_target, dt = _split(oscillator)
    est = LawSynthRegressor(polynomial_degree=1, threshold=0.05, dt=dt, target_name="v")
    est.fit(x_col, v_target)
    assert est.n_features_in_ == 1
    pred = est.predict(x_col)
    series = pred.tolist() if hasattr(pred, "tolist") else pred
    assert len(series) == len(v_target)
    assert est.score(x_col, v_target) > 0.99


def test_equation_names_target(oscillator):
    x_col, v_target, dt = _split(oscillator)
    est = LawSynthRegressor(polynomial_degree=1, dt=dt, target_name="v").fit(x_col, v_target)
    eq = est.equation()
    assert eq.startswith("dv/dt")


def test_target_name_collision(oscillator):
    X, t = oscillator
    x_col = [[row[0]] for row in X]
    v_target = [row[1] for row in X]
    with pytest.raises(ValueError):
        LawSynthRegressor(target_name="x0").fit(x_col, v_target)


def test_requires_y(oscillator):
    x_col, _, _ = _split(oscillator)
    with pytest.raises((ValueError, TypeError)):
        LawSynthRegressor().fit(x_col, None)


def test_length_mismatch(oscillator):
    x_col, v_target, dt = _split(oscillator)
    with pytest.raises(ValueError):
        LawSynthRegressor(dt=dt).fit(x_col, v_target[:-3])


def test_predict_feature_count_guard(oscillator):
    x_col, v_target, dt = _split(oscillator)
    est = LawSynthRegressor(polynomial_degree=1, dt=dt, target_name="v").fit(x_col, v_target)
    with pytest.raises(ValueError):
        est.predict([[1.0, 2.0]])


def test_deterministic(oscillator):
    x_col, v_target, dt = _split(oscillator)
    a = LawSynthRegressor(polynomial_degree=1, dt=dt, target_name="v").fit(x_col, v_target)
    b = LawSynthRegressor(polynomial_degree=1, dt=dt, target_name="v").fit(x_col, v_target)
    assert a.equation() == b.equation()


def test_auto_parsimony(oscillator):
    x_col, v_target, dt = _split(oscillator)
    est = LawSynthRegressor(polynomial_degree=3, parsimony="auto", dt=dt, target_name="v")
    est.fit(x_col, v_target)
    assert hasattr(est, "parsimony_coefficient_")
    assert est.score(x_col, v_target) > 0.99
