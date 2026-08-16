# Units

`NumericColumn::with_unit` stores an optional unit string alongside a numeric
column. The string is included in the dataset fingerprint and is preserved by
normal dataset construction.

The data contract deliberately does not define a unit grammar, conversion, or
dimensional validation rule. The separate `lawsynth-units` crate provides
deterministic SI dimensional-analysis primitives, but attaching a text unit to
a dataset does not automatically parse it or constrain discovery. Consumers
that need dimensional enforcement must validate units before constructing or
running the dataset pipeline.
