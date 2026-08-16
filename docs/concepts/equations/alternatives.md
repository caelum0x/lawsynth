# Candidate alternatives

Discovery fits a sparse feature candidate, then may add a bounded symbolic candidate when `symbolic` configuration is present. It evaluates complexity and mean-squared error, and returns the Pareto front rather than hiding the trade-off in one opaque score.

Feature libraries can include polynomial terms and optional trigonometric or bounded rational terms. The configured state columns receive laws; other columns serve as controls.

Candidates express regression fit under the configured derivative and preprocessing choices. They do not establish a unique governing law.
