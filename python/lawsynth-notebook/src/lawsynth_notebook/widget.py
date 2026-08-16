"""Composable view object; usable in IPython without inheriting ipywidgets."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Any

from .display import RenderedArtifact


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
