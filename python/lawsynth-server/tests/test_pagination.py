import pytest

from lawsynth_server.errors import ValidationError
from lawsynth_server.pagination import page


def test_cursor_pagination_is_stable():
    values = [{"id": str(i)} for i in range(3)]
    first = page(values, cursor=None, limit=2, maximum=10)
    assert [x["id"] for x in first.items] == ["0", "1"]
    assert [x["id"] for x in page(values, cursor=first.next_cursor, limit=2, maximum=10).items] == ["2"]
    with pytest.raises(ValidationError):
        page(values, cursor="bad", limit=2, maximum=10)
