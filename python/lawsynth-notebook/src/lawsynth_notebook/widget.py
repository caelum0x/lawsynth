"""Composable view objects; usable in IPython without inheriting ipywidgets."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .display import RenderedArtifact
from .explorer_view import build_explorer_html
from .serialization import canonical_json


@dataclass(frozen=True, slots=True)
class NotebookWidget:
    view: RenderedArtifact

    @property
    def data(self) -> dict[str, Any]:
        return dict(self.view.data)

    def _repr_html_(self) -> str:
        return self.view.html

    def _repr_mimebundle_(self, **kwargs: object) -> dict[str, str]:
        return self.view._repr_mimebundle_(**kwargs)


@dataclass(frozen=True, slots=True)
class WorldExplorerWidget:
    """An interactive, self-contained explorer for a discovered world.

    Holds the world-as-JSON ``payload`` (states, parameters, parameterised
    laws, initial conditions, time bounds) built by
    :func:`lawsynth_notebook.explorer_payload.build_payload`. Rendering emits a
    single HTML+JS bundle whose embedded integrator re-simulates entirely in the
    browser — no ipywidgets, no comm, no server required. A live-kernel comm is
    an optional enhancement, never a prerequisite for the interaction.
    """

    payload: dict[str, Any]
    theme: str = "light"

    @property
    def data(self) -> dict[str, Any]:
        return dict(self.payload)

    def html(self) -> str:
        """The full self-contained interactive HTML bundle."""
        return build_explorer_html(self.payload, self.theme)

    def _repr_html_(self) -> str:
        return self.html()

    def _repr_mimebundle_(self, **_: object) -> dict[str, str]:
        return {
            "text/html": self.html(),
            "application/json": canonical_json(self.payload),
            "text/plain": repr(self),
        }

    def __repr__(self) -> str:
        states = ", ".join(self.payload.get("states", []))
        count = len(self.payload.get("parameters", []))
        return f"WorldExplorerWidget(name={self.payload.get('name')!r}, states=[{states}], parameters={count})"
