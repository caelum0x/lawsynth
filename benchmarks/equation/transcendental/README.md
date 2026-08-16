# Transcendental algebraic relation

This deterministic P2 benchmark generates observations for `y = sin(x) + 0.25*cos(2*x)`.

## Capability contract

Trigonometric features are available for derivative discovery, but not for a standalone algebraic-response recovery API.

Run `python generate.py --workdir /tmp/transcendental` to materialize the CSV. Run
`python run.py --workdir /tmp/transcendental` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
