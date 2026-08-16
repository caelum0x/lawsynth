from lawsynth.event import Event


def test_event_keeps_typed_crossing_metadata():
    assert Event("threshold", 1.5, "rising").direction == "rising"
