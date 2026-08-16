from lawsynth.uncertainty import Interval


def test_interval_contains_its_closed_bounds():
    interval = Interval(1.0, 3.0)
    assert interval.contains(1.0) and interval.contains(3.0)
