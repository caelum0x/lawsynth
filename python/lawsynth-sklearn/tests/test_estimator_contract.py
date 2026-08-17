"""Core scikit-learn estimator-contract checks (sklearn-free)."""

from __future__ import annotations

import pytest

from lawsynth_sklearn import (
    LawSynthDynamics,
    LawSynthRegressor,
    LawSynthTransformer,
    NotFittedError,
)

ESTIMATORS = [LawSynthDynamics, LawSynthRegressor, LawSynthTransformer]


@pytest.mark.parametrize("cls", ESTIMATORS)
def test_get_params_roundtrip(cls):
    est = cls()
    params = est.get_params()
    rebuilt = cls(**params)
    assert rebuilt.get_params() == params


@pytest.mark.parametrize("cls", ESTIMATORS)
def test_set_params_returns_self_and_mutates(cls):
    est = cls()
    key = next(iter(est.get_params()))
    returned = est.set_params(**{key: est.get_params()[key]})
    assert returned is est


def test_set_params_rejects_unknown():
    est = LawSynthDynamics()
    with pytest.raises(ValueError):
        est.set_params(not_a_real_param=1)


@pytest.mark.parametrize("cls", ESTIMATORS)
def test_param_names_are_sorted_and_stored_verbatim(cls):
    est = cls()
    for name, value in est.get_params(deep=False).items():
        # sklearn contract: __init__ stores each arg unchanged as an attribute.
        assert getattr(est, name) == value


def test_notfitted_dynamics(oscillator):
    X, _ = oscillator
    with pytest.raises(NotFittedError):
        LawSynthDynamics().predict(X)
    with pytest.raises(NotFittedError):
        LawSynthDynamics().equations()


def test_notfitted_regressor(oscillator):
    X, _ = oscillator
    with pytest.raises(NotFittedError):
        LawSynthRegressor().predict([[row[0]] for row in X])


def test_notfitted_transformer(oscillator):
    X, _ = oscillator
    with pytest.raises(NotFittedError):
        LawSynthTransformer().transform(X)


def test_n_features_in_dynamics(oscillator):
    X, t = oscillator
    est = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    assert est.n_features_in_ == 2
    assert est.feature_names_in_ == ("x", "v")


def test_n_features_in_transformer(oscillator):
    X, _ = oscillator
    est = LawSynthTransformer(degree=2).fit(X)
    assert est.n_features_in_ == 2


def test_deterministic_fit_predict(oscillator):
    X, t = oscillator
    a = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    b = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"])
    assert a.equations() == b.equations()
    pa = a.predict(X)
    pb = b.predict(X)
    la = pa.tolist() if hasattr(pa, "tolist") else pa
    lb = pb.tolist() if hasattr(pb, "tolist") else pb
    assert la == lb


@pytest.mark.parametrize("cls", ESTIMATORS)
def test_sklearn_tags(cls):
    est = cls()
    tags = est.__sklearn_tags__()
    assert tags is not None
    # requires_fit is exposed in both the sklearn and standalone tag shapes.
    assert getattr(tags, "requires_fit", True) is True


def test_regressor_estimator_type():
    assert LawSynthRegressor._estimator_type == "regressor"
