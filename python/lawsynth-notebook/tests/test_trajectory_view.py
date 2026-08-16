import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.trajectory_view import render_trajectory


def test_trajectory_is_rendered_and_time_is_strict(fixture_root):
    fixture = __import__("conftest").load_fixture(fixture_root, "trajectory_view")
    assert "Trajectory" in render_trajectory(fixture).html
    with pytest.raises(ArtifactValidationError):
        render_trajectory({"time": [1, 1], "values": {"x": [1, 2]}})
