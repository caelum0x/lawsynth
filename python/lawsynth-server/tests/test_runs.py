import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.runs import RunRepository


def test_run_status_is_constrained():
    assert RunRepository().create("o", {"name": "r"})["status"] == "queued"
    with pytest.raises(ValidationError): RunRepository().create("o", {"name": "r", "status": "unknown"})
