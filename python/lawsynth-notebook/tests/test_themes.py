import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.themes import palette


def test_themes_are_fixed_named_palettes():
    assert palette("dark")["background"] == "#111827"
    with pytest.raises(ArtifactValidationError):
        palette("neon")
