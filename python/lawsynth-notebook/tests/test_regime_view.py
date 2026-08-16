import pytest
from lawsynth_notebook.errors import ArtifactValidationError
from lawsynth_notebook.regime_view import normalize_regimes


def test_regimes_cannot_overlap():
    assert len(normalize_regimes([{"name":"a","start":0,"end":1}])) == 1
    with pytest.raises(ArtifactValidationError):
        normalize_regimes([{"name":"a","start":0,"end":2},{"name":"b","start":1,"end":3}])
