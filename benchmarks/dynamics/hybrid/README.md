# Event-driven hybrid trajectory

This deterministic P2 benchmark generates observations for `piecewise-linear bouncing state with reset at [0,1] bounds`.

## Capability contract

Hybrid interval splitting exists in the simulator, but the public CLI cannot author guard/reset laws or discover hybrid modes from observations.

Run `python generate.py --workdir /tmp/hybrid` to materialize the CSV. Run
`python run.py --workdir /tmp/hybrid` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
