"""LawSynthDynamics — an sklearn-style estimator for governing-equation discovery.

This is the flagship adapter. It wraps the real :class:`lawsynth.Study` discovery
loop in the scikit-learn estimator contract (``fit`` / ``predict`` / ``score`` /
``get_params`` / ``set_params`` / ``__sklearn_tags__`` / ``NotFittedError``), and
extends it with a dynamics-native surface (``simulate``, ``equations``). It is a
*superset* of the static sklearn contract, adapted so that ``X`` is a time-series
whose columns are state variables and ``fit`` recovers the system's evolution
laws rather than a single input→output map.
"""

from __future__ import annotations

from typing import Any, Mapping, Sequence

from lawsynth.study import Study
from lawsynth.trajectory import TrajectoryData

from ._compat import BaseEstimator, NotFittedError, Tags, check_is_fitted, HAS_SKLEARN
from ._data import (
    as_output,
    default_feature_names,
    rows_to_columns,
    time_vector,
    to_float_rows,
)
from ._engine import (
    aligned_column,
    build_config,
    default_step,
    simulate_window,
    trajectory_r2,
)
from ._parsimony import auto_parsimony


def _resolve_names_and_states(
    n_features: int,
    given_names: list[str] | None,
    state: Sequence[str] | None,
) -> tuple[list[str], list[str]]:
    """Resolve column names and the subset to model from ``X`` and ``state``.

    - No ``state``: names come from ``X`` (or ``x0..xk``); all columns modelled.
    - ``state`` naming every column (and ``X`` unnamed): ``state`` *is* the names.
    - ``state`` as a subset of named columns: those columns are modelled.
    """
    if state is None:
        names = default_feature_names(n_features, given_names)
        return names, list(names)

    state_names = [str(s) for s in state]
    if given_names is None:
        if len(state_names) == n_features:
            names = default_feature_names(n_features, state_names)
            return names, list(names)
        raise ValueError(
            "X carries no column names, so `state` must name every column "
            f"({n_features} of them); got {state_names}. Pass a named X "
            "(e.g. a DataFrame) to model a subset."
        )
    names = default_feature_names(n_features, given_names)
    missing = [s for s in state_names if s not in names]
    if missing:
        raise ValueError(f"state names {missing} are not columns of X ({names})")
    return names, state_names


class LawSynthDynamics(BaseEstimator):
    """Discover governing dynamics from a multivariate time-series.

    Parameters mirror :class:`lawsynth.DiscoveryConfig` plus a ``recipe`` shortcut
    and an auto-parsimony switch. Following the sklearn convention, ``__init__``
    only stores its arguments verbatim; all validation and work happen in ``fit``.

    Parameters
    ----------
    polynomial_degree, threshold, solver, derivative_method,
    include_trigonometric, include_rational, smoothing_radius, symbolic_depth
        Forwarded to LawSynth discovery (see :class:`lawsynth.DiscoveryConfig`).
    recipe
        Optional curated preset name (``"mechanics"``, ``"ecology"``, ...). When
        set it supplies the base config and takes precedence over the individual
        knobs, matching the SDK's recipe/config exclusivity.
    parsimony
        ``None`` to use ``threshold`` as given, or ``"auto"`` to select the
        sparsity threshold from a deterministic Cov/Var sweep over a Pareto front
        (see :mod:`lawsynth_sklearn._parsimony`).
    name
        A label carried into explanations/reports.

    Fitted attributes
    -----------------
    ``world_``, ``study_``, ``result_``, ``equations_``, ``states_``,
    ``feature_names_in_``, ``n_features_in_``, ``config_``,
    ``parsimony_coefficient_`` (only when ``parsimony="auto"``).
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
        name: str = "lawsynth-dynamics",
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
        self.name = name

    # -- sklearn tags ------------------------------------------------------- #

    def __sklearn_tags__(self) -> Any:
        if HAS_SKLEARN and hasattr(super(), "__sklearn_tags__"):  # pragma: no cover
            tags = super().__sklearn_tags__()
            tags.non_deterministic = False
            tags.requires_fit = True
            return tags
        return Tags(estimator_type=None, requires_fit=True, non_deterministic=False)

    # -- fit ---------------------------------------------------------------- #

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

    def fit(
        self,
        X: Any,
        y: Any = None,
        *,
        t: Any = None,
        state: Sequence[str] | None = None,
    ) -> "LawSynthDynamics":
        """Discover the world from a time-series ``X``.

        ``X`` is ``(n_samples, n_features)`` with columns as state variables,
        ``t`` an optional time vector (defaults to ``0, 1, ...``), and ``state``
        an optional subset of column names to model (defaults to all columns).
        ``y`` is accepted and ignored so the estimator fits into sklearn tooling.
        """
        rows, given_names = to_float_rows(X)
        n_samples = len(rows)
        n_features = len(rows[0])
        names, state_names = _resolve_names_and_states(n_features, given_names, state)

        times = time_vector(t, n_samples)
        columns = rows_to_columns(rows, names)
        study = Study.from_columns(times, columns, state=state_names, name=self.name)

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
        self.states_ = tuple(state_names)
        self.feature_names_in_ = tuple(names)
        self.n_features_in_ = n_features
        self.config_ = config
        self._time_ = tuple(times)
        return self

    # -- dynamics-native surface -------------------------------------------- #

    def equations(self, *, readable: bool = True) -> dict[str, str]:
        """Discovered laws as ``{state: expression}``.

        With ``readable=True`` returns the plain-language form
        (``dx/dt = 0.999·v``); otherwise the raw native expression.
        """
        check_is_fitted(self)
        if not readable:
            return dict(self.equations_)
        return {law.target: law.readable for law in self.result_.explain().laws}

    def explain(self) -> Any:
        """The full :class:`lawsynth.Explanation` for the discovered world."""
        check_is_fitted(self)
        return self.result_.explain()

    def simulate(
        self,
        *,
        horizon: float | None = None,
        initial: Mapping[str, float] | Sequence[float] | None = None,
        step: float | None = None,
    ) -> TrajectoryData:
        """Forecast a trajectory forward from ``initial`` over ``horizon``.

        ``initial`` may be a ``{state: value}`` mapping or a sequence aligned to
        ``states_``; it defaults to the first observed sample. Returns a
        :class:`lawsynth.trajectory.TrajectoryData` (``.time`` and ``.values``).
        """
        check_is_fitted(self)
        return self.result_.simulate(
            horizon=horizon, initial=self._resolve_initial(initial), step=step
        )

    def _resolve_initial(
        self, initial: Mapping[str, float] | Sequence[float] | None
    ) -> dict[str, float] | None:
        if initial is None:
            return None
        if isinstance(initial, Mapping):
            return {str(k): float(v) for k, v in initial.items()}
        values = list(initial)
        if len(values) != len(self.states_):
            raise ValueError(
                f"initial has {len(values)} values but there are {len(self.states_)} states"
            )
        return {state: float(value) for state, value in zip(self.states_, values)}

    # -- sklearn contract --------------------------------------------------- #

    def predict(self, X: Any) -> Any:
        """Simulate the discovered world seeded from the first row of ``X``.

        Returns a ``(len(X), n_states)`` array of the forecast state trajectory
        on the training sampling grid — a forward roll-out, not a per-row map.
        """
        check_is_fitted(self)
        rows, _ = to_float_rows(X)
        n = len(rows)
        initial = {state: rows[0][self.feature_names_in_.index(state)] for state in self.states_}
        step = default_step(self._time_)
        grid = [self._time_[0] + i * step for i in range(n)]
        trajectory = simulate_window(self.world_, grid, initial, step=step)
        out = [
            [aligned_column(trajectory.values[state], n)[i] for state in self.states_]
            for i in range(n)
        ]
        return as_output(out)

    def score(self, X: Any, t: Any = None) -> float:
        """Mean per-state R² of the simulated vs. observed trajectory of ``X``."""
        check_is_fitted(self)
        rows, _ = to_float_rows(X)
        if len(rows[0]) != self.n_features_in_:
            raise ValueError(
                f"X has {len(rows[0])} features; this estimator was fit on {self.n_features_in_}"
            )
        columns = rows_to_columns(rows, list(self.feature_names_in_))
        times = time_vector(t, len(rows))
        per_state = trajectory_r2(self.world_, times, columns, self.states_)
        return sum(per_state.values()) / len(per_state) if per_state else 0.0
