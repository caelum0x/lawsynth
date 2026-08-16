import pytest

from lawsynth_server.datasets import DatasetRepository
from lawsynth_server.errors import ValidationError


def test_dataset_schema_is_validated():
    repo = DatasetRepository()
    assert repo.create("o", {"name": "d", "schema": ["t", "x"]})["schema"] == ["t", "x"]
    with pytest.raises(ValidationError): repo.create("o", {"name": "bad", "schema": ["x", "x"]})
