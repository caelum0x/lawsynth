# Seeded stochastic process

This deterministic P2 benchmark generates observations for `dX = -1.2 X dt + 0.3 dW (seeded Euler-Maruyama observations)`.

## Capability contract

The engine contains seeded SDE integration internally, but the public discovery CLI does not infer stochastic diffusion terms or accept a stochastic model specification.

Run `python generate.py --workdir /tmp/stochastic` to materialize the CSV. Run
`python run.py --workdir /tmp/stochastic` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
