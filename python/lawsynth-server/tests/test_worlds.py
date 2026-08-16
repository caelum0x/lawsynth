import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.worlds import WorldRepository


def test_world_requires_equations():
    assert WorldRepository().create("o", {"name": "w", "equations": ["dx = x"]})["name"] == "w"
    with pytest.raises(ValidationError): WorldRepository().create("o", {"name": "bad", "equations": []})
