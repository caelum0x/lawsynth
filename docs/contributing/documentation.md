# Documentation contributions

Documentation is part of the public contract. Write from executable behavior:
crate APIs, CLI usage, tests, and checked-in fixtures take precedence over
planned architecture diagrams. Include exact commands and expected artifact
types, and update them when an interface changes.

State capability boundaries plainly. For example, the current core supports
deterministic scalar continuous/discrete Worlds and rejects stochastic,
delayed, regime, causal, custom-operator, and signed-bundle content. Do not
turn a future directory or interface into a product claim.

Run code blocks that can be run. Cross-link the test or example that verifies
a numerical statement. Keep generated data provenance and tolerance details
near scientific documentation. Use Markdown headings in a hierarchy so docs
can be rendered by the repository's static documentation tooling.
