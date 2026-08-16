import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.uncertainty_view import normalize_intervals


def test_uncertainty_rejects_inverted_interval():
    assert normalize_intervals({"k": {"lower": 1, "upper": 2}})[0]["mean"] == 1.5
    with pytest.raises(ArtifactValidationError):
        normalize_intervals({"k": {"lower": 2, "upper": 1}})
