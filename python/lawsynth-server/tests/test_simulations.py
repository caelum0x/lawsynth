import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.simulations import validate_simulation_spec


def test_simulation_spec_has_bounds():
    assert validate_simulation_spec({"horizon": 1, "step": .1})["method"] == "rk4"
    with pytest.raises(ValidationError): validate_simulation_spec({"horizon": 1, "step": 2})
