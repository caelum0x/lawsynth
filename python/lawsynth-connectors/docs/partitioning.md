# Partitioning

`lawsynth_connectors.partitioning` plans deterministic row and time partitions so
large sources can be read in reproducible, evenly sized slices. Planning is pure:
the same inputs always yield the same partitions, independent of any driver.

## Row partitions

```python
from lawsynth_connectors.partitioning import plan_row_partitions

plan_row_partitions(total_rows=250, target_rows=100)
# (RowPartition(0, 0, 100), RowPartition(1, 100, 200), RowPartition(2, 200, 250))
```

Each `RowPartition` is a half-open `[start, stop)` range with a positive `size`;
the final partition absorbs the remainder. `total_rows` may be zero (yielding no
partitions); `target_rows` must be positive.

## Time partitions

```python
from datetime import datetime, timedelta, timezone
from lawsynth_connectors.partitioning import plan_time_partitions

plan_time_partitions(
    datetime(2024, 1, 1, tzinfo=timezone.utc),
    datetime(2024, 1, 3, tzinfo=timezone.utc),
    timedelta(days=1),
)
```

Both bounds must be timezone-aware; they are normalized to UTC before slicing.
Each `TimePartition` is a half-open `[start, stop)` interval, and the last one is
clamped to the requested end. An empty or reversed range, a non-positive
interval, or a naive datetime raises `ValueError`.

## Why it matters

Partitions give connectors and downstream jobs a stable unit of parallelism and
retry. Because boundaries are deterministic, a re-read of the same partition index
produces the same rows or time window, which keeps fingerprints and provenance
consistent across runs.
