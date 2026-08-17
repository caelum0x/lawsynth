# finance-data-adapter

A LawSynth `data.adapter` plugin that normalizes provider-specific market bars
into a single, reproducible **OHLCV** schema. Market-data vendors disagree on
field names, timestamp formats, and ordering; this adapter maps them onto one
canonical shape so LawSynth can discover math worlds from price series.

## Canonical schema

Each output row has: `symbol`, `timestamp`, `open`, `high`, `low`, `close`,
`volume`.

- `timestamp` is normalized to a timezone-aware UTC ISO-8601 string. Epoch
  seconds (int/float) and ISO strings (`Z` accepted) are both handled; naive
  timestamps are rejected.
- `open`, `high`, `low`, `close`, `volume` are coerced to `float`.
- Rows are sorted by timestamp and duplicate timestamps are rejected.
- OHLC integrity is enforced: `low <= min(open, close) <= max(open, close) <= high`.
- `symbol` is upper-cased and must be 1..32 characters.

## Field mapping

Provider field names are supplied through `mapping`, which overrides the default
identity mapping for any of the OHLCV fields:

```python
from finance_data_adapter.plugin import FinanceDataAdapter

adapter = FinanceDataAdapter()
result = adapter.invoke({
    "symbol": "aapl",
    "mapping": {"timestamp": "t", "open": "o", "high": "h",
                "low": "l", "close": "c", "volume": "v"},
    "records": [
        {"t": 1_700_000_000, "o": 10, "h": 12, "l": 9, "c": 11, "v": 1000},
        {"t": 1_700_000_060, "o": 11, "h": 13, "l": 10, "c": 12, "v": 1500},
    ],
})
# result == {"records": [...normalized...], "row_count": 2, "symbol": "AAPL"}
```

## Canonical dataset

Turn a normalized close series into a LawSynth `Dataset`:

```python
rows = result["records"]
time = tuple(i for i, _ in enumerate(rows))          # or parse timestamps to epoch
columns = {"close": tuple(row["close"] for row in rows)}
# lawsynth.dataset.Dataset.from_columns(time, columns)
```

## Install

```bash
pip install -e plugins/finance-data-adapter
```

No runtime dependencies — standard library only.

See [docs/usage.md](docs/usage.md) and [examples/basic.py](examples/basic.py).
