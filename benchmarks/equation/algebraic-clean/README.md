# Clean algebraic polynomial

This deterministic P2 benchmark generates observations for `y = 1 + 2*x - 0.5*x^2`.

## Capability contract

LawSynth currently exposes time-derivative discovery, not a public static algebraic-regression endpoint; this case must not be scored as recovered dynamics.

Run `python generate.py --workdir /tmp/algebraic-clean` to materialize the CSV. Run
`python run.py --workdir /tmp/algebraic-clean` to execute the native path. Results are
written outside the repository so the checked-in fixture remains declarative.
