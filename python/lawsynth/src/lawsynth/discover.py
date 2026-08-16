"""High-level discovery wrapper accepting :class:`Dataset`."""

from collections.abc import Sequence

from .config import DiscoveryConfig
from .dataset import Dataset
from .errors import ValidationError


def discover(dataset: Dataset, states: Sequence[str], config: DiscoveryConfig = DiscoveryConfig()):
    """Discover a native World with fully explicit solver settings."""
    if not states or any(state not in dataset.columns for state in states):
        raise ValidationError("states must name dataset columns")
    from ._native import discover_world
    time, columns = dataset.as_native_arguments()
    return discover_world(
        time, columns, list(states), polynomial_degree=config.polynomial_degree,
        threshold=config.threshold, solver=config.solver,
        include_trigonometric=config.include_trigonometric,
        include_rational=config.include_rational, smoothing_radius=config.smoothing_radius,
        derivative_method=config.derivative_method, symbolic_depth=config.symbolic_depth,
    )
