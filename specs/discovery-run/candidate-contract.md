# Candidate contract

`DiscoveryResult` contains the input `DatasetProfile`, ordered preprocessing
reports, and zero or more retained `DiscoveryCandidate`s. A candidate holds an
executable `lawsynth_world::World`, `CandidateMetrics`, and an optional
bootstrap interval for MSE.

The sparse branch creates one continuous law for every requested state. It
uses all dataset columns as feature variables; selected state columns become
world states and the others become controls. Coefficients below the configured
sparse threshold are omitted, and an all-pruned law becomes the constant zero
expression.

When symbolic search is enabled, one affine-calibrated symbolic candidate may
be added. Final retention uses `pareto_front`: a candidate is discarded only
when another has no greater MSE and complexity and is strictly better in at
least one. Input order among retained candidates is preserved.
