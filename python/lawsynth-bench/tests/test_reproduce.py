from lawsynth_bench.reproduce import fingerprint
def test_fingerprint_is_key_order_independent():
    assert fingerprint({"a": 1, "b": 2}) == fingerprint({"b": 2, "a": 1})
