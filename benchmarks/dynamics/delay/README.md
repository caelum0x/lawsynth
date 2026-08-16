# Delayed feedback

This deterministic P2 benchmark generates observations for `x[t] = 0.8*x[t-1] - 0.25*x[t-5]`.

## Capability contract

The dataset generator is deterministic, but the public discovery CLI does not accept delay coordinates or expose delay identification. It must fail as a declared capability boundary.

Run `python generate.py --workdir /tmp/delay` to materialize the CSV. Run
`python run.py --workdir /tmp/delay` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
