# Datasets

Build validated time-series input with `Dataset.from_columns(time, columns)`. Time values and all numeric values are converted to floats; time must be nonempty, finite, and strictly increasing. There must be at least one column, every column name must be a Python identifier, and each finite column must have the same length as time.

`as_native_arguments()` returns owned `list`/`dict` containers for the extension. The facade handles aligned numeric observations only; it does not read files, infer time, accept missing values, or represent tables with nonnumeric columns.
