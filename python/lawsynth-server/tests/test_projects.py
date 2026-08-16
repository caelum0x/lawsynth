import pytest

from lawsynth_server.errors import ConflictError, NotFoundError
from lawsynth_server.projects import ProjectRepository


def test_project_names_are_unique_per_organization():
    repo = ProjectRepository()
    item = repo.create("a", {"name": "p"})
    with pytest.raises(ConflictError): repo.create("a", {"name": "p"})
    assert repo.create("b", {"name": "p"})["name"] == "p"
    with pytest.raises(NotFoundError): repo.get("b", item["id"])
