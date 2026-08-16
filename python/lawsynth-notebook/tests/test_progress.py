from lawsynth_notebook.progress import Progress


def test_progress_records_audit_trail():
    progress = Progress(2)
    assert progress.advance(message="profile") == .5
    assert progress.history == [(1, "profile")]
