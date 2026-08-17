"""Bridge helpers over the LawSynth SDK shared by the estimators.

Keeps the config-building, native simulation, and trajectory-scoring logic in one
place so :class:`LawSynthDynamics` and :class:`LawSynthRegressor` stay thin and
consistent.
"""

from __future__ import annotations

from dataclasses import replace
from statistics import median
from typing import Mapping, Sequence

from lawsynth import recipes
from lawsynth.config import DiscoveryConfig
from lawsynth.trajectory import TrajectoryData


def build_config(
    *,
    recipe: str | None,
    polynomial_degree: int,
    threshold: float,
    solver: str,
    derivative_method: str,
    include_trigonometric: bool,
    include_rational: bool,
    smoothing_radius: int | None,
    symbolic_depth: int | None,
    threshold_override: float | None = None,
) -> DiscoveryConfig:
    """Materialise a validated :class:`DiscoveryConfig` from estimator params.

    A ``recipe`` (when set) supplies the base config and takes precedence over
    the individual knobs — mirroring the SDK's "recipe or config, not both"
    rule. ``threshold_override`` (used by the auto-parsimony sweep) always wins.
    """
    if recipe is not None:
        config = recipes.get(recipe).config()
        if threshold_override is not None:
            config = replace(config, threshold=threshold_override)
        return config
    resolved_threshold = threshold_override if threshold_override is not None else threshold
    return DiscoveryConfig(
        polynomial_degree=polynomial_degree,
        threshold=resolved_threshold,
        solver=solver,
        derivative_method=derivative_method,
        include_trigonometric=include_trigonometric,
        include_rational=include_rational,
        smoothing_radius=smoothing_radius,
        symbolic_depth=symbolic_depth,
    )


def default_step(time: Sequence[float]) -> float:
    """Median sampling interval; ``1.0`` when fewer than two samples."""
    if len(time) < 2:
        return 1.0
    diffs = [b - a for a, b in zip(time, time[1:])]
    step = float(median(diffs))
    return step if step > 0 else 1.0


def simulate_window(
    world: object,
    time: Sequence[float],
    initial: Mapping[str, float],
    *,
    step: float | None = None,
) -> TrajectoryData:
    """Simulate ``world`` from ``initial`` across the span of ``time``.

    Uses a fixed step (median of the observed intervals unless given) and lands
    ``len(time)`` points on the grid, so simulated series align index-for-index
    with the observations for scoring.
    """
    n = len(time)
    resolved_step = step if step is not None else default_step(time)
    start = float(time[0])
    end = start + (n - 1) * resolved_step
    native = world.simulate(dict(initial), start=start, end=end, step=resolved_step)
    return TrajectoryData.from_native(native)


def _r2(observed: Sequence[float], simulated: Sequence[float]) -> float:
    count = min(len(observed), len(simulated))
    if count == 0:
        return 0.0
    observed = observed[:count]
    simulated = simulated[:count]
    mean = sum(observed) / count
    ss_tot = sum((value - mean) ** 2 for value in observed)
    ss_res = sum((o - s) ** 2 for o, s in zip(observed, simulated))
    if ss_tot == 0.0:
        return 1.0 if ss_res == 0.0 else 0.0
    return 1.0 - ss_res / ss_tot


def trajectory_r2(
    world: object,
    time: Sequence[float],
    columns: Mapping[str, Sequence[float]],
    states: Sequence[str],
    *,
    step: float | None = None,
) -> dict[str, float]:
    """Per-state R² of the simulated trajectory vs. the observed columns."""
    initial = {state: float(columns[state][0]) for state in states}
    trajectory = simulate_window(world, time, initial, step=step)
    return {
        state: _r2(list(columns[state]), list(trajectory.values.get(state, ())))
        for state in states
    }


def aligned_column(values: Sequence[float], n: int) -> list[float]:
    """Force ``values`` to length ``n`` (truncate or pad with the last value)."""
    data = list(values)
    if not data:
        return [0.0] * n
    if len(data) >= n:
        return data[:n]
    return data + [data[-1]] * (n - len(data))
