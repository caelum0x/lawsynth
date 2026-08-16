import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.graph_view import normalize_graph


def test_graph_requires_declared_nodes():
    assert normalize_graph({"x": [], "y": ["x"]})["y"] == ["x"]
    with pytest.raises(ArtifactValidationError):
        normalize_graph({"x": ["missing"]})
