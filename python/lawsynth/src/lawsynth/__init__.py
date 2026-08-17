"""Python interface to the LawSynth executable-world engine.

Importing data-model modules does not require loading the compiled extension;
native classes are resolved lazily when a simulation or discovery is requested.
"""

from . import recipes
from ._version import __version__
from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import LawSynthError, NativeError, ValidationError
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


def __getattr__(name):
    if name in {"Scenario", "Trajectory", "World"}:
        try:
            from . import _native
        except ImportError as error:
            raise NativeError(
                "the lawsynth native extension is unavailable; install the built package"
            ) from error
        return getattr(_native, name)
    if name in {"Study", "DiscoveryResult", "Explanation", "Forecast", "Law", "ScenarioComparison"}:
        from . import study as _study

        return getattr(_study, name)
    raise AttributeError(name)


__all__ = [
    "Dataset", "DiscoveryConfig", "LawSynthError", "NativeError", "ValidationError",
    "SourceError", "load_source", "recipes",
    "profile", "DataProfile", "ColumnProfile", "TimeProfile",
    "discover", "Scenario", "Trajectory", "World",
    "Study", "DiscoveryResult", "Explanation", "Forecast", "Law", "ScenarioComparison",
    "__version__",
]
