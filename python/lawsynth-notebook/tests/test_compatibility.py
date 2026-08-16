import pytest
from lawsynth_notebook.compatibility import check_format
from lawsynth_notebook.errors import ArtifactValidationError


def test_only_current_format_major_is_accepted():
    assert check_format({"format_version": "1.2"}) == 1
    with pytest.raises(ArtifactValidationError):
        check_format({"format_version": 2})
