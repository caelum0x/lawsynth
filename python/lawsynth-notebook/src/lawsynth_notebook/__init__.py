"""Safe, inspectable notebook views for LawSynth artifacts."""

from ._version import __version__
from .config import NotebookConfig
from .display import RenderedArtifact, render_json
from .equation_view import render_equations
from .frontier_view import render_frontier
from .graph_view import render_graph
from .regime_view import render_regimes
from .trajectory_view import render_trajectory
from .uncertainty_view import render_uncertainty
from .widget import NotebookWidget


def render_world(world: dict, config: NotebookConfig | None = None) -> RenderedArtifact:
    """Render a decoded World IR summary without executing its expressions."""
    config = config or NotebookConfig()
    from .compatibility import check_format
    from .serialization import require_mapping
    from .templates import definition_list, panel
    document = require_mapping(world, "world")
    check_format(document)
    states = document.get("states", document.get("variables", []))
    if not isinstance(states, list):
        from .errors import ArtifactValidationError
        raise ArtifactValidationError("world states or variables must be a list")
    fields = {"format version": document.get("format_version", 1), "states / variables": len(states), "parameters": len(document.get("parameters", {})) if isinstance(document.get("parameters", {}), dict) else "unknown"}
    return RenderedArtifact("World", panel("World", definition_list(fields), config.theme), dict(document))


def render_bundle(bundle: dict, config: NotebookConfig | None = None) -> RenderedArtifact:
    """Render decoded bundle manifest metadata; ZIP decoding belongs to the bundle codec."""
    config = config or NotebookConfig()
    from .compatibility import check_format
    from .serialization import require_mapping
    from .templates import definition_list, panel
    document = require_mapping(bundle, "bundle")
    check_format(document)
    fields = {"format version": document.get("format_version", 1), "bundle id": document.get("bundle_id", "not supplied"), "world id": document.get("world_id", "not supplied"), "files": len(document.get("files", [])) if isinstance(document.get("files", []), list) else "unknown"}
    return RenderedArtifact("Bundle", panel("Bundle", definition_list(fields), config.theme), dict(document))


__all__ = ["NotebookConfig", "NotebookWidget", "RenderedArtifact", "__version__", "render_bundle", "render_equations", "render_frontier", "render_graph", "render_json", "render_regimes", "render_trajectory", "render_uncertainty", "render_world"]
