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
