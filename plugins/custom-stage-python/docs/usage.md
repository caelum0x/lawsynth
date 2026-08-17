# Usage: custom-stage-python

A declarative discovery-pipeline stage. Describe transformations as data; the
stage applies them in order, immutably, with no code evaluation.

## Install

```bash
pip install -e plugins/custom-stage-python
```

No third-party dependencies are required.

## Constructing the stage

```python
from custom_stage_python.plugin import CustomStage

stage = CustomStage(max_rows=1_000_000, max_operations=100)
```

Both limits must be positive; exceeding them raises `ValueError`.

## Request

`invoke` accepts:

| key          | type                    | notes                              |
|--------------|-------------------------|------------------------------------|
| `records`    | sequence of mappings    | input rows                         |
| `operations` | sequence of mappings    | ordered transformations            |

`transform(records, operations)` returns just the rows if you do not need the
`{input_rows, output_rows}` envelope.

## Operations

| kind        | fields                         | behaviour                                     |
|-------------|--------------------------------|-----------------------------------------------|
| `select`    | `columns: [str]`               | keep listed columns, in the given order       |
| `rename`    | `mapping: {old: new}`          | rename columns; destinations must be unique   |
| `filter`    | `field, operator, value`       | keep rows where the predicate holds           |
| `fill_null` | `field, value`                 | replace `None` in a column                    |
| `drop_null` | `fields: [str]`                | drop rows with `None` in any listed field     |

`filter` operators: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `in`.
Ordered comparisons treat `None` as "does not match" rather than raising.

## Example

```python
stage.invoke({
    "records": [
        {"t": 0, "x": 1.0}, {"t": 1, "x": None}, {"t": 2, "x": 3.0},
    ],
    "operations": [
        {"kind": "drop_null", "fields": ["x"]},
        {"kind": "rename", "mapping": {"t": "time"}},
        {"kind": "filter", "field": "x", "operator": "gt", "value": 1.0},
    ],
})
# {'records': [{'time': 2, 'x': 3.0}], 'input_rows': 3, 'output_rows': 1}
```

## Guarantees

- **Immutable:** input records are copied, never mutated.
- **Bounded:** `max_rows` and `max_operations` are enforced up front.
- **Safe:** unknown operation kinds and filter operators raise `ValueError`;
  there is no dynamic code path.

## Position in the pipeline

Run this stage after ingestion (for example after `csv-variant-adapter`) and
before building a `lawsynth.dataset.Dataset`, to drop noise columns, filter
regimes, and rename fields into the identifiers discovery expects.
