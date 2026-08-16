# Units and scaling

LawSynth fits numeric values; it does not infer dimensional consistency from column labels. Establish a unit contract before discovery: time units, each state unit, input units, parameter units, and any normalization transform.

Scale variables when their magnitudes differ substantially, but retain the forward and inverse transforms. Coefficients learned in scaled coordinates must be transformed back before scientific interpretation. A useful run record contains the source unit, scale, offset, transformed unit, and equation units.

The Python `Unit` value object validates a small symbolic unit vocabulary for client-side metadata. It is not a full unit algebra and does not prove that a discovered equation is dimensionally valid. Perform domain-specific dimensional review before deployment or publication.
