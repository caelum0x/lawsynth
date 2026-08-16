import pytest
from lawsynth_notebook.controls import RangeControl
from lawsynth_notebook.errors import ArtifactValidationError


def test_range_control_enforces_bounds():
    assert RangeControl("k", 0, 2, 1).value == 1
    with pytest.raises(ArtifactValidationError):
        RangeControl("k", 0, 2, 3)
