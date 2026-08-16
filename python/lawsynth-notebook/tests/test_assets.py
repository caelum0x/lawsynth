import pytest
from lawsynth_notebook.assets import Asset
from lawsynth_notebook.errors import ArtifactValidationError


def test_assets_cannot_escape_bundle_namespace():
    assert Asset("preview.svg", "image/svg+xml", b"x").size == 1
    with pytest.raises(ArtifactValidationError):
        Asset("../secret", "text/plain", b"")
