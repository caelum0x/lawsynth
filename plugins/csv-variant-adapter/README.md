# csv-variant-adapter

A LawSynth `data.adapter` plugin that reads the CSV dialect variants commonly
emitted by scientific and engineering tools and normalizes them into the plain
record form the rest of the LawSynth stack understands.

Real-world CSV is inconsistent: some tools use `;` as the delimiter, a comma for
the decimal point, thousands separators, or a UTF-8 BOM. This adapter absorbs
those differences behind a single bounded entry point so downstream discovery
always sees clean, typed numeric records.

## What it does

- Sniffs the delimiter (`, ; \t |`) or accepts an explicit one.
- Handles comma decimals (`1,5`) and thousands separators (`1.234,5`).
- Decodes with a configurable encoding (default `utf-8-sig`, which strips a BOM).
- Coerces cells to `int`, then `float`, then leaves them as text; blanks become `None`.
- Rejects empty, duplicate, or ragged headers and enforces `max_rows` / `max_bytes`.

It performs **acquisition and structural validation only**. It does not build a
`World`, profile columns, or run discovery — that stays in the core SDK.

## Contract

The plugin exposes a single `invoke(request)` method.

```python
from csv_variant_adapter.plugin import CsvVariantAdapter

adapter = CsvVariantAdapter()
result = adapter.invoke({
    "payload": "time;value\n0;1,5\n1;2,5\n",   # str or bytes
    "options": {"delimiter": ";", "decimal": ","},
})
# result == {
#   "records": [{"time": 0, "value": 1.5}, {"time": 1, "value": 2.5}],
#   "row_count": 2,
#   "columns": ["time", "value"],
# }
```

`options` maps onto the frozen `CsvOptions` dataclass: `delimiter`, `decimal`,
`thousands`, `encoding`, `max_rows`, `max_bytes`. Invalid options raise
`ValueError` before any parsing happens.

## Canonical dataset

The emitted `records` are the aligned numeric form that
`lawsynth.dataset.Dataset.from_columns` consumes. Pick a strictly increasing
time column and the numeric state columns:

```python
time = tuple(row["time"] for row in result["records"])
columns = {"value": tuple(row["value"] for row in result["records"])}
# Dataset.from_columns(time, columns)  # -> ready for discovery
```

## Install

```bash
pip install -e plugins/csv-variant-adapter
```

The package has **no runtime dependencies** — it uses only the standard-library
`csv` module.

## Manifest

`plugin.toml` declares the manifest in the strict grammar from
`specs/plugin-protocol/manifest.md`: id `csv-variant-adapter`, kind `process`,
capability `data.adapter`. Declaring a capability does not grant access; a host
enforces the effective policy.

See [docs/usage.md](docs/usage.md) for a full walkthrough and
[examples/basic.py](examples/basic.py) for a runnable example.
