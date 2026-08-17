"""Deterministic row and time partition planning."""

from __future__ import annotations

from datetime import datetime, timedelta, timezone

import pytest

from lawsynth_connectors.partitioning import (
    RowPartition,
    TimePartition,
    plan_row_partitions,
    plan_time_partitions,
)


def test_row_partition_size_and_validation() -> None:
    part = RowPartition(0, 0, 10)
    assert part.size == 10
    with pytest.raises(ValueError):
        RowPartition(0, 5, 5)
    with pytest.raises(ValueError):
        RowPartition(-1, 0, 1)


def test_plan_row_partitions_covers_range() -> None:
    parts = plan_row_partitions(10, 4)
    assert [(p.start, p.stop) for p in parts] == [(0, 4), (4, 8), (8, 10)]
    assert [p.index for p in parts] == [0, 1, 2]
    assert sum(p.size for p in parts) == 10


def test_plan_row_partitions_empty_and_validation() -> None:
    assert plan_row_partitions(0, 5) == ()
    with pytest.raises(ValueError):
        plan_row_partitions(-1, 5)
    with pytest.raises(ValueError):
        plan_row_partitions(10, 0)


def test_time_partition_requires_tz_aware_bounds() -> None:
    start = datetime(2020, 1, 1, tzinfo=timezone.utc)
    stop = datetime(2020, 1, 2, tzinfo=timezone.utc)
    TimePartition(0, start, stop)
    with pytest.raises(ValueError):
        TimePartition(0, datetime(2020, 1, 1), datetime(2020, 1, 2))
    with pytest.raises(ValueError):
        TimePartition(0, stop, start)


def test_plan_time_partitions_evenly_and_remainder() -> None:
    start = datetime(2020, 1, 1, tzinfo=timezone.utc)
    stop = datetime(2020, 1, 1, 10, tzinfo=timezone.utc)
    parts = plan_time_partitions(start, stop, timedelta(hours=4))
    assert len(parts) == 3
    assert parts[0].start == start
    assert parts[-1].stop == stop
    # partitions are contiguous
    for previous, current in zip(parts, parts[1:]):
        assert previous.stop == current.start


def test_plan_time_partitions_validation() -> None:
    start = datetime(2020, 1, 1, tzinfo=timezone.utc)
    stop = datetime(2020, 1, 2, tzinfo=timezone.utc)
    with pytest.raises(ValueError):
        plan_time_partitions(datetime(2020, 1, 1), stop, timedelta(hours=1))
    with pytest.raises(ValueError):
        plan_time_partitions(stop, start, timedelta(hours=1))
    with pytest.raises(ValueError):
        plan_time_partitions(start, stop, timedelta(0))
