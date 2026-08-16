from lawsynth.run import RunRecord


def test_completed_run_records_utc_timestamp_and_kind():
    record = RunRecord.completed("run-1", "simulation")
    assert record.status == "completed" and record.created_at.tzinfo is not None
