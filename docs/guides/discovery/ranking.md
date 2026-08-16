# Ranking candidates

The engine reports a selected candidate with mean-squared error and complexity. Treat these as diagnostics for the configured fit, not a complete ranking theory. A lower training MSE can be a worse scientific model if it extrapolates poorly or violates known constraints.

Rank runs using a predetermined scorecard: held-out trajectory error, stability over multiple initial conditions, term count, unit consistency, residual structure, and domain review. Store every candidate bundle and score in an external results table so the choice is reproducible.

Current CLI output does not provide a hosted frontier, ensemble ranking service, or calibrated posterior probability. If a workflow needs those artifacts, calculate them in a separately versioned analysis layer and state the method used.
