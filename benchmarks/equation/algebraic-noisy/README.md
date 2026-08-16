# Noisy algebraic polynomial

This deterministic P2 benchmark generates observations for `y = 1 + 2*x - 0.5*x^2 + deterministic Gaussian noise`.

## Capability contract

The native CLI does not expose a static noisy-regression workflow; converting this relation into a derivative target would measure a different problem.

Run `python generate.py --workdir /tmp/algebraic-noisy` to materialize the CSV. Run
`python run.py --workdir /tmp/algebraic-noisy` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
