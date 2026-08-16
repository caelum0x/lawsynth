# Planning a discovery run

Define the scientific target before selecting flags: observed state names, time unit, permitted terms, validation horizon, acceptance metric, and known exclusions. Save this plan alongside the raw-data identifier so hyperparameter changes are reviewable rather than retrofitted after seeing an attractive equation.

Start with a low polynomial degree and a conservative feature family. Fit a baseline, simulate it from observed initial conditions, and compare trajectories on a held-out interval. Increase complexity only when it improves a predefined validation measure and remains interpretable in the stated units.

The CLI writes one selected world bundle. It is not a workflow scheduler, experiment tracker, or automatic search service. Run sweeps through an external orchestrator that records each exact command and preserves every resulting bundle, including rejected candidates.
