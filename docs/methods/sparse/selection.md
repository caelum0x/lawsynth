# Threshold selection

`threshold` controls hard removal for STLSQ/SR3 and the final sparse mask. Larger values favor fewer terms but can discard physically meaningful weak coefficients; small values retain collinear nuisance terms.

LawSynth exposes the parameter rather than fitting it from the same residuals. Select it with a declared validation protocol and compare retained structures, not just in-sample error.
