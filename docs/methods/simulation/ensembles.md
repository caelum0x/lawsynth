# Ensemble boundary

The simulation crate returns one trajectory for one world and request. The SDE helper similarly returns one seeded path per call.

There is no built-in ensemble scheduler, parallel execution policy, trajectory aggregation, or Monte Carlo confidence interval. Callers needing an ensemble must manage seeds, resource limits, and aggregation explicitly.
