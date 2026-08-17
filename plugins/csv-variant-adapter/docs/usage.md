# Usage: csv-variant-adapter

This adapter converts messy real-world CSV into the plain, typed records that the
LawSynth stack consumes. It handles delimiter, decimal, thousands, and encoding
variants behind one bounded `invoke` call.

## Install

```bash
pip install -e plugins/csv-variant-adapter
```

No third-party dependencies are required.

## Request

`invoke` accepts a mapping:

| key       | type          | notes                                                        |
|-----------|---------------|--------------------------------------------------------------|
| `payload` | `str \| bytes`| the CSV document; text is encoded with `encoding`            |
| `encoding`| `str`         | used only when `payload` is text (default `utf-8`)           |
| `options` | `mapping`     | forwarded to `CsvOptions`                                    |

### `CsvOptions`

| field       | default        | meaning                                            |
|-------------|----------------|----------------------------------------------------|
| `delimiter` | `None` (sniff) | one character, or auto-detect from `, ; \t \|`     |
| `decimal`   | `"."`          | `"."` or `","`                                     |
| `thousands` | `None`         | one character stripped before parsing              |
| `encoding`  | `"utf-8-sig"`  | decode encoding (BOM-aware)                         |
| `max_rows`  | `1_000_000`    | reject payloads with more data rows                |
| `max_bytes` | `64 MiB`       | reject payloads larger than this                   |

Invalid options raise `ValueError` before parsing.

## Response

```python
{"records": [{...}, ...], "row_count": int, "columns": [str, ...]}
```

Cells are coerced to `int`, then `float`, otherwise kept as stripped text; blank
cells become `None`.

## Examples

Auto-detected comma CSV:

```python
from csv_variant_adapter.plugin import CsvVariantAdapter

CsvVariantAdapter().invoke({"payload": "time,x\n0,1.5\n1,2.5\n"})
# {'records': [{'time': 0, 'x': 1.5}, {'time': 1, 'x': 2.5}], 'row_count': 2, 'columns': ['time', 'x']}
```

European dialect:

```python
CsvVariantAdapter().invoke({
    "payload": "time;pressure\n0;1.013,25\n",
    "options": {"delimiter": ";", "decimal": ",", "thousands": "."},
})
# {'records': [{'time': 0, 'pressure': 1013.25}], ...}
```

## Feeding discovery

The records are the aligned numeric form for `lawsynth.dataset.Dataset`:

```python
records = CsvVariantAdapter().invoke({"payload": "time,x\n0,1.0\n1,2.0\n"})["records"]
time = tuple(float(r["time"]) for r in records)
columns = {"x": tuple(float(r["x"]) for r in records)}
# Dataset.from_columns(time, columns) then discover a World.
```

## Errors

`ValueError` is raised for: non-unique or empty headers, ragged rows, an encoding
mismatch, and exceeding `max_rows` / `max_bytes`. `TypeError` is raised when
`payload` is neither text nor bytes.
