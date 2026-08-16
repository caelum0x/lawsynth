# Small continuous ODE

This deterministic P2 benchmark generates observations for `dx/dt = -x`.

## Capability contract

The public CLI accepts regularly sampled continuous observations, emits a World bundle, and simulates the discovered bundle.

Run `python generate.py --workdir /tmp/ode-small` to materialize the CSV. Run
`python run.py --workdir /tmp/ode-small` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
