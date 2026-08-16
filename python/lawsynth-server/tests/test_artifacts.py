import pytest

from lawsynth_server.artifacts import artifact_from_bytes
from lawsynth_server.errors import ValidationError


def test_artifact_hash_is_content_addressed():
    assert artifact_from_bytes(b"data", "text/plain").sha256 == artifact_from_bytes(b"data", "text/plain").sha256
    with pytest.raises(ValidationError): artifact_from_bytes(b"")
