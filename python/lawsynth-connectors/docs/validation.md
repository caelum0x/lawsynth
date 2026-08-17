# Validation

`lawsynth_connectors.validation` performs structural record validation without any
dataframe or Arrow dependency. It runs inside `BaseConnector` on every projected
batch when a request asks for it, so invalid data is rejected at the ingestion
boundary rather than deep in a downstream job.

## Schema validation

Describe expected fields with `FieldSpec` and `RecordSchema`:

```python
from lawsynth_connectors.validation import FieldSpec, RecordSchema, validate_records

schema = RecordSchema(
    fields=[
        FieldSpec("time", "number", nullable=False),
        FieldSpec("label", "string"),
    ],
    allow_extra=False,
)
report = validate_records(records, schema)
report.raise_for_errors(connector="filesystem")
```

Supported logical types: `any`, `boolean`, `integer`, `number`, `string`, `date`,
`datetime`. Type checks are strict — `bool` is not an `integer`, non-finite floats
are not a `number`, and `datetime` is not a `date`. A `ReadRequest` with a
`schema` triggers this automatically.

## Report

`validate_records` returns a `ValidationReport` with:

- `valid` — true when there are no issues;
- `issues` — up to `max_issues` `ValidationIssue` records (`row`, `field`, `code`,
  `message`), with codes `required` (a non-nullable field was `None`), `type`
  (value did not match the logical type), and `extra` (an unexpected field when
  `allow_extra=False`);
- `missing_by_field` — a count of `None`/absent values per field.

`raise_for_errors` turns the first issue into a `DataValidationError` carrying the
row, field, and code in redacted details.

## Numeric datasets

`validate_numeric_dataset(records, time_column=...)` enforces the stricter shape
LawSynth discovery needs: a present, strictly usable time column and finite
numeric values. A `ReadRequest` with `numeric=True` runs this check per batch, so
a connector can hand the SDK a dataset that is already known to be dense and
finite.
