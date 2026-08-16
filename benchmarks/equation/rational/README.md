# Rational algebraic relation

This deterministic P2 benchmark generates observations for `y = x / (1 + 0.5*x)`.

## Capability contract

Rational feature flags apply to dynamical derivative discovery. The product has no public static rational-regression endpoint for this benchmark.

Run `python generate.py --workdir /tmp/rational` to materialize the CSV. Run
`python run.py --workdir /tmp/rational` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
