# custom-stage-python

A LawSynth `algorithm` plugin that implements a custom discovery-pipeline stage.
It applies a **bounded, declarative** sequence of record transformations so you
can reshape ingested data before it reaches world discovery — without embedding
arbitrary Python in the pipeline.

There is no `eval`, no `exec`, and no user code path. Every operation is a small
data-only description, which keeps the stage safe to run inside a host that
enforces the plugin protocol's resource limits.

## Supported operations

Each operation is a mapping with a `kind`:

| kind        | fields                          | effect                                        |
|-------------|---------------------------------|-----------------------------------------------|
| `select`    | `columns: [str]`                | keep only these columns, in order             |
| `rename`    | `mapping: {old: new}`           | rename columns (destinations must be unique)  |
| `filter`    | `field, operator, value`        | keep rows where the predicate holds           |
| `fill_null` | `field, value`                  | replace `None` in a column with a default     |
| `drop_null` | `fields: [str]`                 | drop rows with `None` in any listed field     |

`filter` operators: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in`.

The stage enforces `max_rows` and `max_operations` and rejects any unknown
operation kind or filter operator with a `ValueError`.

## Contract

```python
from custom_stage_python.plugin import CustomStage

stage = CustomStage()
result = stage.invoke({
    "records": [
        {"time": 0, "x": 1.0, "scratch": "a"},
        {"time": 1, "x": None, "scratch": "b"},
        {"time": 2, "x": 3.0, "scratch": "c"},
    ],
    "operations": [
        {"kind": "drop_null", "fields": ["x"]},
        {"kind": "select", "columns": ["time", "x"]},
        {"kind": "filter", "field": "time", "operator": "gte", "value": 2},
    ],
})
# result == {
#   "records": [{"time": 2, "x": 3.0}],
#   "input_rows": 3,
#   "output_rows": 1,
# }
```

`transform(records, operations)` is also available directly when you want the
list of rows without the envelope.

## Immutability

The stage never mutates its input. It copies each incoming record and returns
new dictionaries, matching the LawSynth immutable-data convention.

## Install

```bash
pip install -e plugins/custom-stage-python
```

No runtime dependencies — standard library only.

See [docs/usage.md](docs/usage.md) and [examples/basic.py](examples/basic.py).
