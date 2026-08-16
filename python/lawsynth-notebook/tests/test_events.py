from lawsynth_notebook.events import normalize_events


def test_events_are_sorted_reproducibly():
    values = normalize_events([{"time":2,"kind":"b"},{"time":1,"kind":"a"}])
    assert [event["kind"] for event in values] == ["a", "b"]
