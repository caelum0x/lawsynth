"""Deterministic row and time partition planning."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timedelta, timezone


@dataclass(frozen=True, slots=True)
class RowPartition:
    index: int
    start: int
    stop: int

    def __post_init__(self) -> None:
        if self.index < 0 or self.start < 0 or self.stop <= self.start:
            raise ValueError("row partition bounds are invalid")

    @property
    def size(self) -> int:
        return self.stop - self.start


@dataclass(frozen=True, slots=True)
class TimePartition:
    index: int
    start: datetime
    stop: datetime

    def __post_init__(self) -> None:
        if self.index < 0 or self.stop <= self.start:
            raise ValueError("time partition bounds are invalid")
        if self.start.tzinfo is None or self.stop.tzinfo is None:
            raise ValueError("time partition bounds must be timezone-aware")


def plan_row_partitions(total_rows: int, target_rows: int) -> tuple[RowPartition, ...]:
    if total_rows < 0:
        raise ValueError("total_rows cannot be negative")
    if target_rows < 1:
        raise ValueError("target_rows must be positive")

    return tuple(
        RowPartition(index, start, min(start + target_rows, total_rows))
        for index, start in enumerate(range(0, total_rows, target_rows))
    )


def plan_time_partitions(
    start: datetime,
    stop: datetime,
    interval: timedelta,
) -> tuple[TimePartition, ...]:
    if start.tzinfo is None or stop.tzinfo is None:
        raise ValueError("time bounds must be timezone-aware")
    if stop <= start:
        raise ValueError("time range must be non-empty")
    if interval <= timedelta(0):
        raise ValueError("partition interval must be positive")

    normalized_start = start.astimezone(timezone.utc)
    normalized_stop = stop.astimezone(timezone.utc)
    partitions: list[TimePartition] = []
    cursor = normalized_start

    while cursor < normalized_stop:
        boundary = min(cursor + interval, normalized_stop)
        partitions.append(TimePartition(len(partitions), cursor, boundary))
        cursor = boundary
    return tuple(partitions)
