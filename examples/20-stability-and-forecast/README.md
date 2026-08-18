# Frictionless order book — stability, invariants, and forecast

This scenario goes one step past discovery: it discovers a world from a
deterministic dataset and then **analyses** it with the shipped engine — locating
and classifying fixed points (`stability`), recovering conserved quantities
(`invariants`), and rolling the model forward (`forecast`). Every step runs a
real, shipped `lawsynth` subcommand.

## The system

An undamped mid-price / order-flow oscillator (a limit-order book with zero
resilience), stated in [config.toml](config.toml):

```
d(mid)/dt       =  impact · imbalance          (impact = 1)
d(imbalance)/dt = -liquidity · mid             (liquidity = 4, resilience = 0)
```

With no resilience the field is **conservative**. It has a single fixed point at
the origin, and that fixed point is a **center**: trajectories are closed orbits
that neither grow nor decay. A center is precisely the case a linearization
cannot decide — its Jacobian eigenvalues are purely imaginary — so `stability`
reports it as *inconclusive*. `invariants` then explains why, by recovering the
conserved energy `liquidity·mid² + impact·imbalance²` whose level sets are those
orbits.

## Run it

The base workflow mirrors every other scenario (generate → discover → simulate),
validated by the executable contract:

```bash
python generate.py
python discover.py
python simulate.py
python -m pytest test_example.py
```

The analysis walkthrough chains discovery and the engine's analysis commands. It
locates the compiled CLI the way the benchmarks do (and builds it once if
missing), then discovers the world and analyses it:

```bash
python analyze.py
```

## What `analyze.py` prints (real transcript)

```
# Frictionless order book — dynamics analysis
An undamped mid-price / order-flow oscillator (zero resilience): a conservative center.

[1] dataset: examples/20-stability-and-forecast/output/observations.csv (1001 samples)
    engine: target/debug/lawsynth

[2] discover -> examples/20-stability-and-forecast/output/analysis-world.lsworld
    discovered world: mse=2.546119e-18, complexity=6
    dimbalance/dt = -3.998933 * mid
    dmid/dt = 0.999733 * imbalance

[3] stability (fixed points + linear classification)
  search: 25/25 seeds converged inside the box (state order: imbalance, mid)
  fixed point (+0.0000, +0.0000) -> center (marginal, inconclusive) [INCONCLUSIVE]
    Jacobian eigenvalues: +0.0000-1.9995i, +0.0000+1.9995i

[4] invariants (conserved quantities, degree-2 monomial library)
  Conserved quantity(ies): 1
  #1  H = 0.25·imbalance^2 + 1.00·mid^2
  residual:       0.000000e0
  singular value: 1.360665e-15

[5] forecast (roll the discovered world forward)
  wrote forecast: .../output/forecast.csv (13 rows)
  forecast horizon t in [0, 6], 13 samples
  final state:
  imbalance        0 -> 1.10472
  mid              1 -> 0.747411

done: stability found a center (inconclusive), invariants recovered its conserved energy, forecast stayed on a bounded orbit.
```

Reading the results:

- **Discovery** recovers the linear field almost exactly (`mse ≈ 2.5e-18`). The
  `-3.998933` and `0.999733` coefficients are `-liquidity` and `impact`; the tiny
  offsets from `-4` and `1` are RK4 truncation, not model error.
- **Stability** locates the origin and classifies it as `center (marginal,
  inconclusive)`. The eigenvalues `±1.9995i` are purely imaginary
  (`≈ ±√(impact·liquidity) i = ±2i`). This is not a failure — it is the engine
  being honest that a linearization cannot decide a non-hyperbolic point.
- **Invariants** recovers `H = 0.25·imbalance² + 1.00·mid²`, i.e.
  `mid² + 0.25·imbalance²`, which is `liquidity·mid² + impact·imbalance²` rescaled.
  Its residual is `0` (Lie derivative `L_f H = 0` on the sample grid). This
  conserved energy is the reason the fixed point is a center.
- **Forecast** rolls the discovered world forward on a bounded, closed orbit —
  consistent with the conserved energy: no growth, no decay.

## Both invocation styles

`analyze.py` shells out to the CLI (robust with no package install). The typed
Python SDK reads the same world and returns parsed dataclasses — this is the real
output of `lawsynth.analysis.stability` on the discovered world:

```python
from lawsynth.analysis import stability
rep = stability("output/analysis-world.lsworld", box="-3:3,-6:6")
fp = rep.fixed_points[0]
# states           : ('imbalance', 'mid')
# seeds converged  : 25 / 25
# fixed point      : (0.0, 0.0)
# classification   : center (marginal, inconclusive)
# inconclusive     : True
# eigenvalues      : [(0.0, -1.9995), (0.0, 1.9995)]
```

Note that `invariants` is a CLI-only command (it is not part of the
`lawsynth.analysis` SDK surface), so the walkthrough uses the CLI for it.

## Honest limits

- The data are synthetic and deterministic; they exercise the pipeline, not a
  causal identification from observational data.
- The **center** is reported as *inconclusive* by design: linear stability cannot
  decide non-hyperbolic equilibria. The conserved quantity is what resolves the
  qualitative picture.
- `invariants` is **library-bounded**: it only finds conserved quantities
  expressible in its degree-`D` monomial basis (here degree 2, plus `--trig` for
  sinusoids). An empty result means "none in this library", not "none exists".
- `bifurcation` and `sensitivity` are intentionally **not** shown here: they need
  a world with *declared parameters*, and a discovered world inlines its
  coefficients as constants, so the engine (correctly) rejects a parameter sweep
  on it. Stability, invariants, and forecast are the analyses that fit a
  discovered, autonomous world.
