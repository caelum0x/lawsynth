# Binary split

`best_binary_split` evaluates every valid single breakpoint and returns the one with largest reduction in within-range SSE, including its left and right costs. If no split can satisfy two minimum-length segments it returns `None`.

This is a one-split diagnostic, not recursive binary segmentation. A positive gain is not a calibrated change-point test.
