"""Python interface to the LawSynth executable-world engine.

Importing data-model modules does not require loading the compiled extension;
native classes are resolved lazily when a simulation or discovery is requested.
"""

from . import recipes
from ._version import __version__
from .client import Client, Run
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import ApiError, LawSynthError, NativeError, RunTimeout, ValidationError
from .prepare import preprocess
from .profile import ColumnProfile, DataProfile, TimeProfile, profile
from .sources import SourceError, load_source


def discover(
    time,
    columns,
    *,
    state,
    polynomial_degree=2,
    threshold=0.05,
    solver="stlsq",
    include_trigonometric=False,
    include_rational=False,
    smoothing_radius=None,
    derivative_method="finite",
    savgol_window=5,
    tvreg_lambda=0.1,
    tvreg_iterations=100,
    symbolic_depth=None,
):
    """Discover a continuous World from aligned numeric observations.

    ``columns`` maps names to sequences aligned with ``time``; ``state`` names
    the columns whose derivatives should be modeled. Optional trigonometric and
    bounded rational feature families can be enabled independently. Derivative
    methods include ``finite``, ``savgol``, ``spline``, ``spectral`` (periodic
    regular grids), and ``tvreg``.
    """
    from ._native import discover_world

    return discover_world(
        list(time),
        {name: list(values) for name, values in columns.items()},
        list(state),
        polynomial_degree=polynomial_degree,
        threshold=threshold,
        solver=solver,
        include_trigonometric=include_trigonometric,
        include_rational=include_rational,
        smoothing_radius=smoothing_radius,
        derivative_method=derivative_method,
        savgol_window=savgol_window,
        tvreg_lambda=tvreg_lambda,
        tvreg_iterations=tvreg_iterations,
        symbolic_depth=symbolic_depth,
    )


def simplify(world):
    """Simplify every law of ``world`` into a canonical, equivalent form.

    Returns a :class:`~lawsynth.simplification.SimplifiedWorld` report holding
    the new equivalent world, per-law before/after expressions and AST node
    counts, and a :meth:`~lawsynth.simplification.SimplifiedWorld.verify` method
    that simulates both worlds and returns the maximum trajectory deviation.
    """
    from . import simplification

    return simplification.simplify_world(world)


def compose(world_a, world_b, *, prefix_a="", prefix_b=""):
    """Combine two worlds into one coupled system (union of states/params/laws).

    Colliding identifiers are namespaced; ``prefix_a`` / ``prefix_b`` prefix all
    of a world's identifiers. Returns a valid native ``World`` that simulates.
    """
    from . import composition

    return composition.compose(world_a, world_b, prefix_a=prefix_a, prefix_b=prefix_b)


def __getattr__(name):
    if name in {"Scenario", "Trajectory", "World"}:
        try:
            from . import _native
        except ImportError as error:
            raise NativeError(
                "the lawsynth native extension is unavailable; install the built package"
            ) from error
        # Ensure editing/simplification methods are attached to the native World.
        from . import composition, simplification  # noqa: F401
        return getattr(_native, name)
    if name in {"Study", "DiscoveryResult", "Explanation", "Forecast", "Law", "ScenarioComparison"}:
        from . import study as _study

        return getattr(_study, name)
    if name in {"backtest", "Backtest", "OriginResult"}:
        from . import backtesting as _backtesting

        return getattr(_backtesting, name)
    if name in {"Project", "ProjectEntry"}:
        from . import project as _project

        return getattr(_project, name)
    if name in {"Ensemble", "ForecastBand", "TermStat"}:
        from . import ensemble as _ensemble

        return getattr(_ensemble, name)
    if name in {"SimplifiedWorld", "LawSimplification", "simplify_world"}:
        from . import simplification as _simplification

        return getattr(_simplification, name)
    if name in {"WorldSpec", "spec_of"}:
        from . import worldspec as _worldspec

        return getattr(_worldspec, name)
    if name in {"monitor", "MonitorReport", "StateResidual", "Anomaly"}:
        # Import via importlib: the submodule shares the name ``monitor`` with the
        # function, so ``from . import monitor`` would re-enter this hook.
        import importlib

        _monitor = importlib.import_module(".monitor", __name__)
        return getattr(_monitor, name)
    if name in {"validate", "Validation"}:
        import importlib

        _validation = importlib.import_module(".validation", __name__)
        return getattr(_validation, name)
    if name in {"model_card", "ModelCard"}:
        from . import governance as _governance

        return getattr(_governance, name)
    if name in {"Lineage", "LineageLink"}:
        from . import lineage as _lineage

        return getattr(_lineage, name)
    if name in {"stream_discover", "StreamHistory", "StreamModel", "ChangeRecord", "TermChange"}:
        from . import streaming as _streaming

        return getattr(_streaming, name)
    if name in {"AuditLog", "AuditEntry"}:
        from . import audit as _audit

        return getattr(_audit, name)
    raise AttributeError(name)


__all__ = [
    "Dataset", "DiscoveryConfig", "LawSynthError", "NativeError", "ValidationError",
    "ApiError", "RunTimeout", "Client", "Run",
    "SourceError", "load_source", "recipes",
    "profile", "DataProfile", "ColumnProfile", "TimeProfile",
    "preprocess",
    "simplify", "compose", "SimplifiedWorld", "LawSimplification", "WorldSpec", "spec_of",
    "discover", "Scenario", "Trajectory", "World",
    "Study", "DiscoveryResult", "Explanation", "Forecast", "Law", "ScenarioComparison",
    "backtest", "Backtest", "OriginResult",
    "Project", "ProjectEntry",
    "Ensemble", "ForecastBand", "TermStat",
    "monitor", "MonitorReport", "StateResidual", "Anomaly",
    "stream_discover", "StreamHistory", "StreamModel", "ChangeRecord", "TermChange",
    "validate", "Validation",
    "model_card", "ModelCard", "Lineage", "LineageLink", "AuditLog", "AuditEntry",
    "__version__",
]
