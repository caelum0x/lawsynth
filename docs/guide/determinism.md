# Determinism

Determinism is LawSynth's core differentiator. Discovery is a fixed, seeded
computation: the same observations and the same configuration produce a
**byte-identical** world bundle, every time, on any machine — no wall-clock
reads, no unseeded randomness, no network. This is what makes a discovered world
citable and auditable rather than a one-off result you can never reproduce.

The claim below is exactly what the tool guarantees and nothing more; the formal
contract is the [`reproducibility`](../../specs/reproducibility) spec (data hash,
plan hash, algorithm version, seed plan, environment).

## Demonstrate it: two runs, one file

Run discovery twice into different output paths, then compare the bytes:

```sh
lawsynth discover lotka-volterra.csv --time time --state x,y --preset ecology --output a.lsworld
lawsynth discover lotka-volterra.csv --time time --state x,y --preset ecology --output b.lsworld
shasum -a 256 a.lsworld b.lsworld
cmp a.lsworld b.lsworld && echo "BYTE-IDENTICAL"
```

```
7fdde728d3ecd68a6f6f0687e2a393e1006894c022740314f48c2d0fc081a18c  a.lsworld
7fdde728d3ecd68a6f6f0687e2a393e1006894c022740314f48c2d0fc081a18c  b.lsworld
BYTE-IDENTICAL
```

The two bundles hash to the same digest and `cmp` reports no differences. The
`run_all.sh` verification script performs exactly this `cmp` check and fails if
the two runs ever diverge.

> The digest above is for this exact input (the shipped 200-sample
> `lotka-volterra.csv`) and configuration (`--preset ecology`) built from the
> current workspace. Change the data, any flag, or the algorithm version and the
> digest changes — deliberately, because the world changed.

## Simulation is deterministic too

`simulate` and `forecast` print trajectories at full 17-digit precision so
downstream tooling can diff them exactly:

```sh
lawsynth simulate a.lsworld --initial x=10 --initial y=5 --start 0 --end 2 --step 0.5
```

```
time,x,y
0.00000000000000000e0,1.00000000000000000e1,5.00000000000000000e0
5.00000000000000000e-1,5.65127591640327775e0,6.03519841161347159e0
1.00000000000000000e0,2.88374771443967415e0,6.06930302659370557e0
1.50000000000000000e0,1.56084342004881460e0,5.52822818892534595e0
2.00000000000000000e0,9.61564303591452840e-1,4.81134687372857694e0
```

Re-running the same command yields identical text. `forecast --confidence` takes
an explicit `--seed`, so even its bootstrap band is reproducible.

## What is (and isn't) guaranteed

- **Guaranteed:** identical inputs + identical config + identical algorithm
  version + identical binary → byte-identical `.lsworld` and identical printed
  trajectories. Verified here by `shasum`/`cmp` and by `run_all.sh`.
- **Not claimed:** cross-version stability. A different LawSynth version may
  change the algorithm and therefore the bytes; the
  [`reproducibility`](../../specs/reproducibility) spec versions the algorithm so
  a digest is always interpreted against a known version. Floating-point results
  can also differ across fundamentally different hardware/math libraries; the
  spec documents a hardware class for this reason.

## Self-validating domain presets

The curated domain presets are a built-in, deterministic round-trip: synthesize a
textbook law's clean trajectory, discover from it, and report per-state RHS RMSE
against the reference.

```sh
lawsynth domains run damped-oscillator
```

```
Round-trip recovery for preset 'damped-oscillator'
  Damped linear harmonic oscillator: dx/dt = v, dv/dt = -x - 0.5 v.

Discovered law(s):
  dv/dt = -0.999987 * x + -0.499985 * v
  dx/dt = 0.999983 * v

Recovery vs. reference (clean trajectory):
  x            RHS RMSE=3.7257e-6  terms 1/1 (discovered/reference)  -> recovered
  v            RHS RMSE=3.3642e-6  terms 2/2 (discovered/reference)  -> recovered

Recovery: OK (every state within RMSE tolerance 1e-3 on clean data).
Note: round-trip is on clean synthetic data; it validates the search space, not noise robustness.
```

This is deterministic (no RNG, no clock) and doubles as a self-test: the preset
recovers its own reference law on clean data. The closing note is the honest
caveat — a good round-trip validates that the preset's search space contains the
reference law, not that discovery is robust to real measurement noise. See
[`domain-packs`](../../specs/domain-packs).
