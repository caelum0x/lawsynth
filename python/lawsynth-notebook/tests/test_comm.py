from lawsynth_notebook.comm import LocalComm


def test_local_comm_copies_messages_for_subscribers():
    comm, seen = LocalComm(), []
    comm.subscribe(seen.append)
    comm.send({"kind": "ready"})
    assert seen == [{"kind": "ready"}] and comm.messages == seen
