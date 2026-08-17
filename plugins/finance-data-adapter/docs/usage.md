# Usage: finance-data-adapter

Normalize provider-specific market bars into one reproducible OHLCV schema so
LawSynth can discover math worlds from price series.

## Install

```bash
pip install -e plugins/finance-data-adapter
```

No third-party dependencies are required.

## Constructing the adapter

```python
from finance_data_adapter.plugin import FinanceDataAdapter

adapter = FinanceDataAdapter(max_rows=1_000_000)
```

## Request

| key       | type                  | notes                                              |
|-----------|-----------------------|----------------------------------------------------|
| `symbol`  | `str`                 | 1..32 chars; upper-cased in the output             |
| `records` | sequence of mappings  | vendor bars                                        |
| `mapping` | mapping               | overrides field names for any OHLCV field          |

The default `mapping` is the identity map over
`timestamp, open, high, low, close, volume`. Supply overrides only for the
fields your vendor names differently.

## Output schema

Each row: `symbol, timestamp, open, high, low, close, volume`.

- `timestamp`: timezone-aware UTC ISO-8601. Epoch seconds (int/float) or ISO
  strings (trailing `Z` accepted) are supported; naive timestamps are rejected.
- OHLCV numeric fields are coerced to `float`.
- Rows are sorted by timestamp; duplicate timestamps are rejected.
- OHLC integrity is enforced:
  `low <= min(open, close) <= max(open, close) <= high`.

## Response

```python
{"records": [{...}, ...], "row_count": int, "symbol": str}
```

## Example

```python
adapter.invoke({
    "symbol": "aapl",
    "mapping": {"timestamp": "t", "open": "o", "high": "h",
                "low": "l", "close": "c", "volume": "v"},
    "records": [
        {"t": 1_704_153_600, "o": 100, "h": 102, "l": 99, "c": 101, "v": 1000},
    ],
})
# {'records': [{'symbol': 'AAPL', 'timestamp': '2024-01-02T00:00:00+00:00',
#              'open': 100.0, 'high': 102.0, 'low': 99.0, 'close': 101.0,
#              'volume': 1000.0}], 'row_count': 1, 'symbol': 'AAPL'}
```

## Feeding discovery

A normalized close series maps onto a `lawsynth.dataset.Dataset` column:

```python
rows = adapter.invoke({...})["records"]
time = tuple(float(i) for i, _ in enumerate(rows))   # or convert timestamps to epoch
columns = {"close": tuple(r["close"] for r in rows)}
# Dataset.from_columns(time, columns)
```

## Errors

`ValueError` for: an invalid symbol, missing OHLC data, OHLC-bound violations,
duplicate timestamps, naive timestamps, or exceeding `max_rows`. `TypeError`
when a record is not a mapping.
