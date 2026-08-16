# Chaotic Lorenz ODE

This deterministic P2 benchmark generates observations for `dx=sigma(y-x), dy=x(rho-z)-y, dz=xy-beta*z`.

## Capability contract

The public CLI supports multivariate continuous derivative discovery and simulation; this is an execution/reproducibility benchmark, not an assertion of exact long-horizon chaotic recovery.

Run `python generate.py --workdir /tmp/ode-chaotic` to materialize the CSV. Run
`python run.py --workdir /tmp/ode-chaotic` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
