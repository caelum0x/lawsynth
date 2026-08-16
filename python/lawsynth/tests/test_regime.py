from lawsynth.regime import RegimeInterval, RegimeSchedule


def test_regime_schedule_sorts_intervals_and_uses_half_open_membership():
    schedule = RegimeSchedule((RegimeInterval("late", 2, 3), RegimeInterval("early", 0, 2)))
    assert schedule.active_at(2).name == "late"
    assert schedule.active_at(3) is None
