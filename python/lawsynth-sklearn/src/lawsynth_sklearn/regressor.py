"""LawSynthRegressor — a strict sklearn RegressorMixin over LawSynth discovery.

LawSynth is dynamics-first: it discovers *evolution laws* (derivatives), not a
static input→output map. This estimator gives the faithful static-regression
framing that drops into ``Pipeline`` / ``GridSearchCV``:

- ``fit(X, y)`` treats ``X`` as the observed trajectories of predictor state
  variables and ``y`` as the observed trajectory of a single target variable. It
  discovers the *coupled* governing world over ``[predictors..., target]`` and
  keeps the law for ``d(target)/dt``.
- ``predict(X)`` seeds that world from the first row of ``X`` (plus the target's
  fitted initial condition) and rolls it forward, returning the reconstructed
  target trajectory — i.e. "what the discovered law says the target does."
- ``score(X, y)`` is the standard R² of that reconstruction against ``y``
  (inherited from :class:`RegressorMixin`).

This is a deliberate, documented subset: rows must be time-ordered because the
target is governed by a differential law, not an i.i.d. regression surface.
"""

from __future__ import annotations

from typing import Any, Sequence

from lawsynth.study import Study

from ._compat import (
    BaseEstimator,
    HAS_SKLEARN,
    RegressorMixin,
    Tags,
    check_is_fitted,
)
from ._data import (
    as_output_1d,
    default_feature_names,
    rows_to_columns,
    time_vector,
    to_float_rows,
)
from ._engine import aligned_column, build_config, default_step, simulate_window
from ._parsimony import auto_parsimony


class LawSynthRegressor(RegressorMixin, BaseEstimator):
    """Discover a governing law for a single target series from predictors.

    Parameters mirror :class:`LawSynthDynamics`, plus ``target_name`` (the
    identifier used for the ``y`` column inside the discovered world) and ``dt``
    (the sampling interval when no explicit time is supplied to ``fit``).
    """

    def __init__(
        self,
        *,
        polynomial_degree: int = 2,
        threshold: float = 0.05,
        solver: str = "stlsq",
        derivative_method: str = "finite",
        include_trigonometric: bool = False,
        include_rational: bool = False,
        smoothing_radius: int | None = None,
        symbolic_depth: int | None = None,
        recipe: str | None = None,
        parsimony: str | None = None,
        target_name: str = "target",
        dt: float = 1.0,
        name: str = "lawsynth-regressor",
    ) -> None:
        self.polynomial_degree = polynomial_degree
        self.threshold = threshold
        self.solver = solver
        self.derivative_method = derivative_method
        self.include_trigonometric = include_trigonometric
        self.include_rational = include_rational
        self.smoothing_radius = smoothing_radius
        self.symbolic_depth = symbolic_depth
        self.recipe = recipe
        self.parsimony = parsimony
        self.target_name = target_name
        self.dt = dt
        self.name = name

    def __sklearn_tags__(self) -> Any:
        if HAS_SKLEARN and hasattr(super(), "__sklearn_tags__"):  # pragma: no cover
            tags = super().__sklearn_tags__()
            tags.non_deterministic = False
            tags.requires_fit = True
            return tags
        return Tags(estimator_type="regressor", requires_fit=True, non_deterministic=False)

    def _config_kwargs(self) -> dict[str, Any]:
        return {
            "recipe": self.recipe,
            "polynomial_degree": self.polynomial_degree,
            "threshold": self.threshold,
            "solver": self.solver,
            "derivative_method": self.derivative_method,
            "include_trigonometric": self.include_trigonometric,
            "include_rational": self.include_rational,
            "smoothing_radius": self.smoothing_radius,
            "symbolic_depth": self.symbolic_depth,
        }

    def fit(self, X: Any, y: Any, *, t: Any = None) -> "LawSynthRegressor":
        """Discover the coupled world for ``[predictors..., target]``.

        ``X`` is ``(n_samples, n_features)`` predictor trajectories, ``y`` the
        aligned ``(n_samples,)`` target trajectory. Rows are time-ordered.
        """
        rows, given_names = to_float_rows(X)
        if y is None:
            raise ValueError("LawSynthRegressor.fit requires y (the target series)")
        target_rows, _ = to_float_rows(y, allow_1d=True)
        target = [row[0] for row in target_rows]
        if len(target) != len(rows):
            raise ValueError("X and y must have the same number of samples")

        n_features = len(rows[0])
        predictor_names = default_feature_names(n_features, given_names)
        if self.target_name in predictor_names:
            raise ValueError(
                f"target_name {self.target_name!r} collides with a predictor column"
            )
        if not self.target_name.isidentifier():
            raise ValueError(f"target_name {self.target_name!r} must be an identifier")

        times = time_vector(t if t is not None else self.dt, len(rows))
        columns = rows_to_columns(rows, predictor_names)
        columns[self.target_name] = target
        all_states = [*predictor_names, self.target_name]
        study = Study.from_columns(times, columns, state=all_states, name=self.name)

        if self.parsimony == "auto":
            def _discover(threshold: float) -> object:
                config = build_config(**self._config_kwargs(), threshold_override=threshold)
                return study.discover(config)

            outcome = auto_parsimony(_discover, self.threshold)
            self.parsimony_coefficient_ = outcome.parsimony_coefficient
            self.parsimony_candidates_ = outcome.candidates
            config = build_config(**self._config_kwargs(), threshold_override=outcome.threshold)
        elif self.parsimony is not None:
            raise ValueError(f"parsimony must be None or 'auto', got {self.parsimony!r}")
        else:
            config = build_config(**self._config_kwargs())

        result = study.discover(config)

        self.study_ = study
        self.result_ = result
        self.world_ = result.world
        self.equations_ = dict(result.equations)
        self.feature_names_in_ = tuple(predictor_names)
        self.states_ = tuple(all_states)
        self.n_features_in_ = n_features
        self.config_ = config
        self._time_ = tuple(times)
        self._initial_target_ = float(target[0])
        return self

    def equation(self, *, readable: bool = True) -> str:
        """The discovered law for the target derivative."""
        check_is_fitted(self)
        for law in self.result_.explain().laws:
            if law.target == self.target_name:
                return law.readable if readable else law.expression
        return self.equations_.get(self.target_name, "")

    def predict(self, X: Any) -> Any:
        """Roll the discovered world forward and return the target trajectory."""
        check_is_fitted(self)
        rows, _ = to_float_rows(X)
        if len(rows[0]) != self.n_features_in_:
            raise ValueError(
                f"X has {len(rows[0])} features; this estimator was fit on {self.n_features_in_}"
            )
        n = len(rows)
        initial = {name: rows[0][i] for i, name in enumerate(self.feature_names_in_)}
        initial[self.target_name] = self._initial_target_
        step = default_step(self._time_)
        grid = [self._time_[0] + i * step for i in range(n)]
        trajectory = simulate_window(self.world_, grid, initial, step=step)
        series = aligned_column(trajectory.values.get(self.target_name, ()), n)
        return as_output_1d(series)
