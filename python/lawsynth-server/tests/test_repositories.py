import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.repositories import Repository


def test_repository_soft_deletes_and_rejects_immutable_updates():
    repo = Repository("thing")
    item = repo.create("o", {"name": "x"})
    with pytest.raises(ValidationError): repo.update("o", item["id"], {"organization_id": "other"})
    repo.delete("o", item["id"])
    assert repo.list("o") == []
