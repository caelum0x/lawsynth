import pytest

from lawsynth_server.errors import NotFoundError
from lawsynth_server.storage import FileObjectStore


def test_filesystem_store_deduplicates_content(tmp_path):
    store = FileObjectStore(tmp_path, max_bytes=10)
    artifact = store.put(b"hello", "text/plain")
    assert store.get(artifact.sha256) == b"hello"
    with pytest.raises(NotFoundError): store.get("0" * 64)
