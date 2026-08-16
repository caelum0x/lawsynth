import pytest

from lawsynth_server.errors import IdempotencyConflict
from lawsynth_server.idempotency import IdempotencyStore


def test_replay_and_payload_conflict():
    store, calls = IdempotencyStore(), []
    def handler():
        calls.append(1)
        return 201, {"id": str(len(calls))}
    assert store.execute("org", "key", {"a": 1}, handler)[2] is False
    assert store.execute("org", "key", {"a": 1}, handler)[2] is True
    assert len(calls) == 1
    with pytest.raises(IdempotencyConflict):
        store.execute("org", "key", {"a": 2}, handler)
