# Time order

`validate_time_order` requires a nonempty vector of finite, strictly increasing timestamps and returns its start, end, and number of observations. Equal or reversed times are rejected with their offending index.

This establishes ordering only. It does not establish a sampling rate, alignment between series, or a causal direction.
