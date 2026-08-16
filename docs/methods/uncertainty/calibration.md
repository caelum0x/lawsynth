# Calibration boundary

LawSynth does not automatically calibrate interval coverage, bootstrap bias, or nominal confidence against reference experiments. `IntervalConfig` only validates a requested central confidence level in `(0, 1)`.

Coverage is a property of a statistical procedure and data-generating assumptions, not a label that can be supplied by configuration alone.
