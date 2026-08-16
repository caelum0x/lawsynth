# Comparing simulations

Use a shared scenario definition, horizon, step, and output schema when comparing bundles or parameter settings. Align results by physical time before calculating error, peak timing, integral quantities, or threshold crossings.

Keep model selection data separate from final evaluation data. A useful report shows the baseline and alternative trajectories, their initial conditions and overrides, numerical convergence checks, and the metric definition—not only a visual overlay.

The CLI emits trajectory CSV but does not implement model-comparison statistics, plotting, or automatic acceptance rules. Compute those in a reviewed analysis layer and store enough metadata to reproduce the comparison.
