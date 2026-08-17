"""LawSynth × scikit-learn quickstart.

Run (offline, deterministic)::

    PYTHONPATH="python/lawsynth-sklearn/src:python/lawsynth/src" \
        python3 python/lawsynth-sklearn/examples/sklearn_quickstart.py

Demonstrates:
  (a) LawSynthDynamics recovering a harmonic oscillator and forecasting it.
  (b) sklearn interop — Pipeline + clone + get_params — when sklearn is present;
      otherwise the standalone estimator contract is exercised instead.
  (c) LawSynthTransformer emitting an engineered feature library.
  (d) parsimony='auto' selecting a sparsity threshold via a Cov/Var Pareto sweep.
"""

from __future__ import annotations

import math

import lawsynth_sklearn as ls
from lawsynth_sklearn import (
    HAS_SKLEARN,
    LawSynthDynamics,
    LawSynthRegressor,
    LawSynthTransformer,
)


def make_oscillator(n: int = 160, dt: float = 0.05) -> tuple[list[list[float]], list[float]]:
    """A clean harmonic oscillator x'' = -x  ->  x = cos t, v = -sin t."""
    t = [i * dt for i in range(n)]
    X = [[math.cos(ti), -math.sin(ti)] for ti in t]  # columns: [x, v]
    return X, t


def section(title: str) -> None:
    print("\n" + "=" * 70)
    print(title)
    print("=" * 70)


def main() -> None:
    print(f"lawsynth_sklearn {ls.__version__}  |  sklearn present: {HAS_SKLEARN}")
    X, t = make_oscillator()

    # (a) Dynamics recovery + forecast ------------------------------------- #
    section("(a) LawSynthDynamics: recover a governing law and forecast")
    dyn = LawSynthDynamics(polynomial_degree=1, threshold=0.05, name="oscillator")
    dyn.fit(X, t=t, state=["x", "v"])
    print("Discovered laws:")
    for target, readable in sorted(dyn.equations().items()):
        print(f"    {readable}")
    print(f"n_features_in_ = {dyn.n_features_in_}, states = {dyn.states_}")
    print(f"in-sample trajectory R² (score) = {dyn.score(X, t):.6f}")

    forecast = dyn.simulate(horizon=2.0, initial={"x": 1.0, "v": 0.0})
    x_pred = forecast.values["x"]
    x_true = [math.cos(ti) for ti in forecast.time]
    max_err = max(abs(a - b) for a, b in zip(x_pred, x_true))
    print(f"forecast from (x=1, v=0) over horizon 2.0: {len(forecast.time)} points, "
          f"max |x_pred - cos t| = {max_err:.4f}")

    # (d) auto-parsimony ---------------------------------------------------- #
    section("(d) parsimony='auto': Cov/Var complexity price over a Pareto sweep")
    auto = LawSynthDynamics(polynomial_degree=3, parsimony="auto", name="auto")
    auto.fit(X, t=t, state=["x", "v"])
    print(f"Cov(complexity, loss)/Var(complexity) = {auto.parsimony_coefficient_:.6g}")
    print(f"selected threshold = {auto.config_.threshold}")
    print("Pareto sweep (threshold | complexity | loss | on_front | penalized):")
    for c in auto.parsimony_candidates_:
        star = "*" if c.on_front else " "
        print(f"    {c.threshold:<8g} {c.complexity:<3d} {c.loss:<10.6f} {star}  {c.penalized:.6f}")
    print("Laws chosen by auto-parsimony:")
    for target, readable in sorted(auto.equations().items()):
        print(f"    {readable}")

    # (c) Transformer ------------------------------------------------------- #
    section("(c) LawSynthTransformer: engineered feature library")
    trans = LawSynthTransformer(degree=2, include_bias=True, include_trigonometric=True)
    Z = trans.fit_transform(X)
    print(f"emitted {len(trans.get_feature_names_out())} features from {trans.n_features_in_} inputs:")
    print("   ", list(trans.get_feature_names_out()))
    row0 = Z[0] if not hasattr(Z, "tolist") else Z[0].tolist()
    print("first engineered row:", [round(float(v), 4) for v in row0])

    pruned = LawSynthTransformer(degree=2, include_bias=True, prune_correlation=True,
                                 correlation_threshold=0.98)
    pruned.fit(X)
    print(f"correlation-pruned subset ({len(pruned.feature_names_out_)} cols):",
          list(pruned.feature_names_out_))

    # (b) sklearn interop --------------------------------------------------- #
    section("(b) sklearn interop")
    if HAS_SKLEARN:
        from sklearn import clone
        from sklearn.linear_model import LinearRegression
        from sklearn.pipeline import Pipeline

        # clone + get_params round-trip on the dynamics estimator
        cloned = clone(dyn)
        assert cloned.get_params() == dyn.get_params()
        print("clone(dyn).get_params() == dyn.get_params():", True)

        # Regressor inside a Pipeline behind the feature transformer.
        x_col = [[row[0]] for row in X]          # predictor: x
        v_target = [row[1] for row in X]         # target: v  (v' = -x)
        pipe = Pipeline([
            ("features", LawSynthTransformer(degree=1)),
            ("law", LawSynthRegressor(polynomial_degree=1, threshold=0.05,
                                      dt=0.05, target_name="v")),
        ])
        # NOTE: transformer here just passes x through (degree=1, no bias) so the
        # regressor sees the predictor column; fit/predict/score all flow.
        pipe.fit(x_col, v_target)
        print("Pipeline[transformer -> LawSynthRegressor] fitted.")
        print(f"    discovered target law: {pipe.named_steps['law'].equation()}")
        print(f"    pipeline R² = {pipe.score(x_col, v_target):.6f}")

        # A transformer + LinearRegression pipeline (downstream sklearn model).
        lin = Pipeline([
            ("lib", LawSynthTransformer(degree=2, include_bias=False)),
            ("lr", LinearRegression()),
        ])
        lin.fit(X, v_target)
        print(f"    TransformerMixin -> LinearRegression R² = {lin.score(X, v_target):.6f}")
    else:
        print("sklearn not installed — exercising the STANDALONE estimator contract:")
        # get_params / set_params round-trip
        params = dyn.get_params()
        rebuilt = LawSynthDynamics(**params)
        assert rebuilt.get_params() == params
        print("    get_params/set_params round-trip: OK")
        rebuilt.set_params(threshold=0.1, polynomial_degree=2)
        assert rebuilt.threshold == 0.1 and rebuilt.polynomial_degree == 2
        print("    set_params mutation: OK")

        # standalone Regressor fit/predict/score
        x_col = [[row[0]] for row in X]
        v_target = [row[1] for row in X]
        reg = LawSynthRegressor(polynomial_degree=1, threshold=0.05, dt=0.05, target_name="v")
        reg.fit(x_col, v_target)
        print(f"    LawSynthRegressor law: {reg.equation()}")
        print(f"    LawSynthRegressor R² (standalone RegressorMixin.score): "
              f"{reg.score(x_col, v_target):.6f}")

        # NotFittedError before fit
        try:
            LawSynthDynamics().predict(X)
        except ls.NotFittedError:
            print("    NotFittedError raised before fit: OK")

        # determinism: two identical fits produce identical equations
        a = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"]).equations()
        b = LawSynthDynamics(polynomial_degree=1).fit(X, t=t, state=["x", "v"]).equations()
        print(f"    deterministic (two fits identical): {a == b}")

    print("\nDone.")


if __name__ == "__main__":
    main()
