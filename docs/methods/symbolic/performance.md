# Search limits and cost

Enumeration stops at `max_candidates` and expands at most `max_depth` frontier rounds. Canonical fingerprints avoid duplicate retained expressions. E-graph normalization is bounded by `max_passes`; cost is scalar AST node count.

These limits are correctness and resource controls, not evidence that a search has exhausted the space of plausible laws.
