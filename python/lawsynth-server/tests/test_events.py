from lawsynth_server.events import EventJournal


def test_events_are_append_only_and_tenant_scoped():
    journal = EventJournal()
    one = journal.append("a", "run.created", {"id": "1"})
    journal.append("b", "run.created", {"id": "2"})
    assert journal.list("a")[0]["event_id"] == one.event_id
    assert journal.list("b")[0]["payload"]["id"] == "2"
