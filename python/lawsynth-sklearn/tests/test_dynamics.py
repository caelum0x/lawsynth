"""LawSynthDynamics: discovery, forecasting, scoring, auto-parsimony."""

from __future__ import annotations

import math

import pytest

from lawsynth_sklearn import LawSynthDynamics


def test_recovers_oscillator(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1, threshold=0.05).fit(X, t=t, state=["x", "v"])
    eqs = est.equations()
    assert set(eqs) == {"x", "v"}
    # x' = v (positive ~1), v' = -x (negative ~-1)
    assert "v" in eqs["x"] and "0.99" in eqs["x"]
    assert "x" in eqs["v"] and "-0.99" in eqs["v"]


def test_score_is_high_on_clean_data(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    assert est.score(X, t) > 0.99


def test_predict_shape_and_seed(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    pred = est.predict(X)
    rows = pred.tolist() if hasattr(pred, "tolist") else pred
    assert len(rows) == len(X)
    assert len(rows[0]) == 2
    # Seeded from the first observed row (x=1, v=0).
    assert abs(rows[0][0] - 1.0) < 1e-6
    assert abs(rows[0][1] - 0.0) < 1e-6


def test_simulate_forecast_matches_cosine(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    traj = est.simulate(horizon=2.0, initial={"x": 1.0, "v": 0.0})
    err = max(abs(v - math.cos(ti)) for ti, v in zip(traj.time, traj.values["x"]))
    assert err < 0.01


def test_simulate_accepts_sequence_initial(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    traj = est.simulate(horizon=1.0, initial=[1.0, 0.0])
    assert len(traj.time) > 1


def test_equations_readable_and_raw(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    readable = est.equations(readable=True)
    raw = est.equations(readable=False)
    assert readable != raw
    assert all("dt" in v for v in readable.values())


def test_recipe_takes_precedence(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(recipe="mechanics").fit(X, t=t, state=["x", "v"])
    # mechanics is a cubic (degree 3) recipe.
    assert est.config_.polynomial_degree == 3


def test_default_state_names_when_unnamed(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t)
    assert est.states_ == ("x0", "x1")


def test_state_must_name_all_unnamed_columns(oscillator):
    X, t = oscillator
    with pytest.raises(ValueError):
        LawSynthDynamics().fit(X, t=t, state=["only_one"])


def test_auto_parsimony_selects_and_reports(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=3, parsimony="auto").fit(X, t=t, state=["x", "v"])
    assert hasattr(est, "parsimony_coefficient_")
    assert len(est.parsimony_candidates_) >= 2
    assert any(c.on_front for c in est.parsimony_candidates_)
    # Auto-parsimony still recovers the clean linear structure.
    assert est.score(X, t) > 0.99


def test_auto_parsimony_is_deterministic(oscillator):
    X, t = oscillator
    a = LawSynthDynamics(polynomial_degree=3, parsimony="auto").fit(X, t=t, state=["x", "v"])
    b = LawSynthDynamics(polynomial_degree=3, parsimony="auto").fit(X, t=t, state=["x", "v"])
    assert a.config_.threshold == b.config_.threshold
    assert a.parsimony_coefficient_ == b.parsimony_coefficient_


def test_invalid_parsimony_value(oscillator):
    X, t = oscillator
    with pytest.raises(ValueError):
        LawSynthDynamics(parsimony="bogus").fit(X, t=t, state=["x", "v"])


def test_score_rejects_wrong_feature_count(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    with pytest.raises(ValueError):
        est.score([[1.0]], None)
