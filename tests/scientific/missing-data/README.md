# Missing-data boundary

This case inserts actual `nan` fields into an otherwise valid growth CSV and
runs the native `discover` command. The command must reject the non-finite
observations. The result is intentionally not a fake imputation benchmark:
automatic missing-data policy is not currently exposed by this CLI path.
