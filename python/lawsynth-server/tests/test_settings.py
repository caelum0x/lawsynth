import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.settings import Settings


def test_settings_reject_invalid_limit():
    with pytest.raises(ValidationError):
        Settings(max_page_size=0)
