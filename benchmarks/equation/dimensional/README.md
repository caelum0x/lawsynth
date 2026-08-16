# Dimension-aware acceleration

This deterministic P2 benchmark generates observations for `position = 0.5*a*t^2; velocity = a*t, a = 9.81 m/s^2`.

## Capability contract

The public discovery command has no dataset-unit input or dimensional-equation scoring API, so a unit-sensitive recovery score would be invented.

Run `python generate.py --workdir /tmp/dimensional` to materialize the CSV. Run
`python run.py --workdir /tmp/dimensional` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
