import pytest
from lawsynth_notebook.errors import UnsupportedCapabilityError
from lawsynth_notebook.server_proxy import LocalArtifactProxy, connect


def test_proxy_is_explicitly_local_only():
    assert LocalArtifactProxy({"world": {"format_version": 1}}).get("world")["format_version"] == 1
    with pytest.raises(UnsupportedCapabilityError):
        connect("https://example.invalid")
