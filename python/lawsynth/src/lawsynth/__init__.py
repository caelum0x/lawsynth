"""Python interface to the LawSynth executable-world engine."""

from ._native import Scenario, Trajectory, World, discover_world as _discover_world


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
    return _discover_world(
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


__all__ = ["discover", "Scenario", "Trajectory", "World"]
