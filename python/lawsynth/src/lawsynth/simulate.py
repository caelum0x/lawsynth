"""Simulation wrapper returning immutable Python trajectory data."""

from .scenario import Scenario
from .trajectory import TrajectoryData


def simulate(world: object, scenario: Scenario, *, start: float = 0.0, end: float = 1.0, step: float = 0.01) -> TrajectoryData:
    """Run a native world with values and scheduled interventions from a scenario."""
    parameters = [(item.time, item.target, item.value) for item in scenario.interventions if item.kind == "parameter"]
    inputs = [(item.time, item.target, item.value) for item in scenario.interventions if item.kind == "input"]
    from ._native import Scenario as NativeScenario
    request = NativeScenario(world, dict(scenario.initial), dict(scenario.parameters), dict(scenario.inputs), parameters, inputs)
    return TrajectoryData.from_native(request.simulate(start=start, end=end, step=step))
