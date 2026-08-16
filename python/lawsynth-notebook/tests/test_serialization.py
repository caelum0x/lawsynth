import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.serialization import canonical_json, finite_number


def test_canonical_json_is_stable_and_rejects_nan():
    assert canonical_json({"b": 1, "a": 2}) == '{"a":2,"b":1}'
    with pytest.raises(ArtifactValidationError):
        finite_number(float("nan"), "x")
