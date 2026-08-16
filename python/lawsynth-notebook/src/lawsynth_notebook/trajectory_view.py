"""Trajectory validation and compact tabular rendering."""

from __future__ import annotations

from collections.abc import Mapping, Sequence
from typing import Any

from .config import NotebookConfig
from .display import RenderedArtifact
from .errors import ArtifactValidationError
from .serialization import finite_number
from .templates import panel, table


def normalize_trajectory(trajectory: Mapping[str, Any]) -> dict[str, Any]:
    if not isinstance(trajectory, Mapping):
        raise ArtifactValidationError("trajectory must be an object")
    time = trajectory.get("time")
    values = trajectory.get("values")
    if not isinstance(time, Sequence) or isinstance(time, (str, bytes)) or not isinstance(values, Mapping) or not time:
        raise ArtifactValidationError("trajectory needs non-empty time and values")
    normalized_time = [finite_number(value, "time") for value in time]
    if any(right <= left for left, right in zip(normalized_time, normalized_time[1:])):
        raise ArtifactValidationError("trajectory time must strictly increase")
    normalized_values: dict[str, list[float]] = {}
    for name, series in values.items():
        if not isinstance(name, str) or not isinstance(series, Sequence) or isinstance(series, (str, bytes)) or len(series) != len(time):
            raise ArtifactValidationError("each trajectory series must align with time")
        normalized_values[name] = [finite_number(value, f"values.{name}") for value in series]
    if not normalized_values:
        raise ArtifactValidationError("trajectory needs at least one series")
    return {"time": normalized_time, "values": dict(sorted(normalized_values.items()))}


def render_trajectory(trajectory: Mapping[str, Any], config: NotebookConfig | None = None) -> RenderedArtifact:
    config = config or NotebookConfig()
    normalized = normalize_trajectory(trajectory)
    names = list(normalized["values"])
    limit = min(config.max_rows, config.max_series_points)
    indices = list(range(min(len(normalized["time"]), limit)))
    rows = [[normalized["time"][i], *[normalized["values"][name][i] for name in names]] for i in indices]
    suffix = "" if len(indices) == len(normalized["time"]) else f"<p>Showing first {len(indices)} of {len(normalized['time'])} samples.</p>"
    return RenderedArtifact("Trajectory", panel("Trajectory", table(["time", *names], rows) + suffix, config.theme), normalized)
