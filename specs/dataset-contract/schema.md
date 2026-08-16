# Schema

A LawSynth dataset has exactly one time axis and at least one scalar numeric
measurement column. `DatasetSchema` is the lexicographically ordered list of
measurement `Identifier`s; the time axis is structural and is not included in
that list.

Each `NumericColumn` contains an `Identifier`, `Vec<f64>`, and an optional
unit string. Identifier validation is performed by `lawsynth_core::Identifier`.
Duplicate identifiers are rejected. Every column must have precisely the same
row count as the time axis, and every measurement must be finite.

Schema order is canonicalized through a `BTreeMap`. Callers may supply columns
in any order, but readers see sorted identifiers. No nested fields, strings,
categoricals, vectors, nullable values, or per-row unit changes exist in this
contract.
