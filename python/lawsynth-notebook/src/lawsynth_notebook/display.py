"""IPython-compatible but dependency-free rendered values."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .serialization import canonical_json
from .templates import panel, text


@dataclass(frozen=True, slots=True)
class RenderedArtifact:
    title: str
    html: str
    data: dict[str, Any]

    def _repr_html_(self) -> str:
        return self.html

    def _repr_mimebundle_(self, **_: object) -> dict[str, str]:
        return {"text/html": self.html, "application/json": canonical_json(self.data), "text/plain": repr(self)}

    def __repr__(self) -> str:
        return f"RenderedArtifact(title={self.title!r}, keys={sorted(self.data)})"


def render_json(title: str, data: dict[str, Any], theme: str = "light") -> RenderedArtifact:
    raw = canonical_json(data)
    return RenderedArtifact(title, panel(title, f"<pre>{text(raw)}</pre>", theme), data)


def render_world_object(world: Any, theme: str = "light") -> RenderedArtifact:
    """Render a live SDK/native World (anything exposing ``.equations()``)."""
    from .equation_view import render_equations

    equations = world.equations() if callable(getattr(world, "equations", None)) else getattr(world, "equations", None)
    return render_equations(dict(equations), theme)


def render_trajectory_object(trajectory: Any, theme: str = "light") -> RenderedArtifact:
    """Render a live SDK/native Trajectory (with ``.time`` and ``.values``)."""
    from .config import NotebookConfig
    from .trajectory_view import render_trajectory

    payload = {"time": list(trajectory.time), "values": {name: list(series) for name, series in trajectory.values.items()}}
    return render_trajectory(payload, NotebookConfig(theme=theme))


def render_law_object(obj: Any, theme: str = "light") -> RenderedArtifact:
    """Dispatch a live LawSynth SDK object to the matching notebook view."""
    if callable(getattr(obj, "equations", None)) or isinstance(getattr(obj, "equations", None), dict):
        return render_world_object(obj, theme)
    if hasattr(obj, "time") and hasattr(obj, "values"):
        return render_trajectory_object(obj, theme)
    from .errors import ArtifactValidationError

    raise ArtifactValidationError("object is not a renderable LawSynth world or trajectory")


def render_study_dashboard(source: Any, theme: str = "light", **kwargs: Any) -> Any:
    """Compose a cohesive :class:`~lawsynth_notebook.dashboard.StudyDashboard`.

    ``source`` is a live LawSynth ``Study`` or ``DiscoveryResult`` (anything
    exposing ``name``, ``states``, ``explain()`` and ``simulate()``).
    """
    from .dashboard import render_dashboard

    return render_dashboard(source, theme=theme, **kwargs)
