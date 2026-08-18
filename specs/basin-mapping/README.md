# Basin-of-attraction mapping boundary (v2-A)

This directory specifies **deterministic basin-of-attraction mapping** — the
global question of *which initial conditions flow to which stable attractor* of a
discovered vector field, implemented in `crates/lawsynth-basins`. It is a
**boundary specification** in the house style: it states what a conforming
implementation MUST do, and — crucially — what a basin map is and is not allowed
to claim.

## Motivation

Stability analysis finds a discovered law's fixed points and classifies each
**locally** (stable node/spiral, saddle, …). Bifurcation analysis tracks how that
local picture changes as a parameter moves. Neither answers the **global**
question: given several coexisting stable attractors, *from where* does the system
reach each one?

That question is the **basin of attraction** — the set of initial conditions
whose forward trajectory converges to a given attractor. Basins partition state
space (up to the boundary sets between them) and are what decides the long-term
fate of a system started from a particular state. A bistable switch, a double-well
memory, a species that survives or collapses depending on its starting population
— all are basin questions.

LawSynth is **deterministic and offline**. This stage **reuses** the fixed-point
and linear-stability analysis of `crates/lawsynth-stability` to find the
attractors, and the `crates/lawsynth-expr` IR to evaluate the field; the only
local numerics are a fixed-step RK4 forward flow. Identical inputs MUST yield
bit-identical output.

## What a basin map IS

A basin map is a **finite, deterministic sampling** of the flow's long-term fate,
read off a **fixed grid** of initial conditions. The contract is:

1. **Attractors are the stable fixed points, found not proven exhaustive.** The
   attractor set is exactly the STABLE fixed points located by
   `analyze_stability` inside the search box — stable nodes and stable spirals.
   Saddles, unstable points, and non-hyperbolic (`Center`/`Marginal`) points are
   **not** attractors and are never mapped. If no stable fixed point is found, the
   report is honestly empty rather than inventing a basin.
2. **Labels are the flow's verdict, not a nearest-attractor guess.** Each initial
   condition is integrated **forward in time**; it is labelled with an attractor
   only if its trajectory actually comes within a tolerance of that attractor. A
   trajectory that leaves the box or diverges is `Escaped`; one that does neither
   within `max_time` is `Undetermined`. Classification is never forced.
3. **A property of the field, not of any data.** The mapping integrates whatever
   field it is handed; it carries no discovery confidence or fit residual. Those
   are separate, upstream concerns.
4. **Honest about the sampling.** The report records the grid resolution, the box,
   and the per-attractor settled fractions alongside the `escaped` and
   `undetermined` counts, so a lopsided or empty result reads as "this is what the
   grid at this resolution found", not "this is the exact basin geometry".

## Requirements

1. **Attractors from stability.** The attractor set MUST be obtained by
   delegating to `lawsynth_stability::analyze_stability` over the configured
   `StabilityConfig`, then retaining exactly the fixed points classified
   `StableNode` or `StableSpiral`, in the order stability reports them
   (lexicographic by coordinate). The attractor index used in labels is the
   position in that filtered, ordered list.

2. **Deterministic initial-condition grid.** Initial conditions MUST be a fixed,
   content-independent even lattice over the search box: `resolution` samples per
   axis, enumerated as a Cartesian product in a fixed row-major order (first axis
   varies slowest). The reported `grid_labels` MUST follow exactly that order.
   Unlike the Newton seed set, the origin is NOT specially appended — the grid is
   precisely the lattice.

3. **Fixed-step RK4 forward flow.** Each initial condition MUST be integrated with
   the classical fourth-order Runge–Kutta method at a fixed step `dt`, evaluating
   the field with the `lawsynth-expr` evaluator. The number of steps is
   `ceil(max_time / dt)`. The arithmetic MUST be performed in a fixed order so the
   flow is bit-reproducible.

4. **Endpoint classification with honest outcomes.** A trajectory MUST be labelled:
   - `Attractor(i)` if at any checked step it comes within `convergence_tolerance`
     (measured in the `‖·‖∞` / Chebyshev metric) of attractor `i`; the nearest
     attractor wins, ties resolve to the lowest index;
   - `Escaped` if it leaves the search box padded by `escape_margin` on every
     axis, exceeds `divergence_limit` in magnitude, becomes non-finite, or the
     field is undefined along the step (e.g. `log` of a non-positive argument);
   - `Undetermined` if neither happens within `max_time`.

   An implementation MUST NOT coerce an `Escaped` or `Undetermined` trajectory
   into a basin.

5. **Fractions over the settled population.** The per-attractor `fractions` MUST be
   `count(attractor i) / settled_total`, where `settled_total` is the number of
   trajectories labelled with some attractor (escaped and undetermined excluded).
   If nothing settled, every fraction MUST be `0.0`. The `escaped` and
   `undetermined` counts MUST be reported separately.

6. **Determinism.** Attractor detection, grid generation, RK4 integration, and
   classification MUST be deterministic. Identical `(fields, states, BasinConfig)`
   inputs MUST produce a **bit-identical** `BasinReport`: identical attractor
   coordinates (to `f64` bit patterns), identical fractions (to bit patterns), and
   identical labels.

7. **Autonomy and totality.** The field MUST be autonomous: every symbol it
   references MUST be one of the states, otherwise there is no value to integrate
   at and the implementation MUST return a typed error (surfaced through the
   stability layer as `UnknownSymbol`). Structural faults (dimension mismatch
   between states and box, inverted search interval, out-of-range scalar knob,
   empty state space) MUST surface as distinct typed errors — never a silently
   dropped or fabricated result.

## Public API

```text
map_basins(&[(Identifier, Expr)], &[Identifier], &BasinConfig)
    -> Result<BasinReport, BasinError>

BasinConfig::new(search_box) -> Self              // + with_* builder setters
BasinReport {
    states, attractors, grid_labels, fractions,
    escaped, undetermined, resolution, search_box,
}
Attractor { coordinates, classification }
Label = Attractor(usize) | Escaped | Undetermined
BasinReport::to_canonical_string() -> String      // determinism fingerprint
```

`BasinConfig` carries the search box (for both the IC grid and the escape
region), the grid resolution, the RK4 `dt`, the `max_time`, the
`convergence_tolerance`, the `escape_margin` and `divergence_limit`, and the
`StabilityConfig` used to find the attractors. This crate delivers the **basin
mapping library** only; visualising, refining boundaries, or wiring it into a
discovery report is downstream and out of scope here.

## Honest scope & limits

- **Only STABLE fixed-point attractors are recognized.** This is the most
  important limit. A **limit cycle** or a **strange (chaotic) attractor** is a
  perfectly real attractor, but it is not a fixed point, so `analyze_stability`
  does not find it and this crate does not classify its basin. Initial conditions
  that converge to such an attractor stay bounded without approaching any listed
  fixed point, so they read as `Undetermined` (or `Escaped` if they wander past
  the box). The report is honest about this — it never labels those trajectories
  with a fixed-point basin — but it does **not** map the true attractor. Deciding
  those basins needs periodic-orbit or Poincaré-section machinery this crate does
  not perform.
- **Resolution bounds boundary accuracy.** Basin boundaries (the stable manifolds
  of the intervening saddles) can be intricate, even fractal for forced or
  chaotic systems. A finite grid can only resolve them to its spacing; a point
  near a boundary may fall on either side. A finer grid resolves more, at
  `resolution^n` cost. Nothing here proves a basin's exact geometry.
- **A fixed `max_time` may under-settle slow systems.** A trajectory that is still
  drifting toward an attractor when the clock runs out is reported
  `Undetermined`, not mislabelled. Slowly-converging (e.g. weakly-damped spiral)
  systems need a larger `max_time` and/or a looser `convergence_tolerance`; the
  implementation exposes both and MUST NOT silently extend the horizon to force a
  verdict.
- **Fixed-step RK4 carries truncation error.** The flow is a fourth-order
  approximation at step `dt`; stiff or fast fields need a smaller `dt`. The
  approximation is deterministic but not exact, and the crate performs no adaptive
  step control or error estimation.
- **The map is box-bounded.** The IC grid, the escape test, and the attractor
  search all live inside the search box. Attractors outside the box are not found,
  and a trajectory that exits the box is `Escaped` even if it would have converged
  to something further out. Widening the box sees more, at more cost.
- **Supported functions are exactly the IR's** (`+ − × ÷`, `^`, negate, `exp`,
  `log`, `sin`, `cos`), inherited from `lawsynth-expr`; a field outside that set
  cannot be integrated.

## Non-goals

- No limit-cycle, periodic-orbit, or strange-attractor detection or basin
  mapping; only STABLE fixed-point attractors are recognized.
- No adaptive-step or stiff integration, no trajectory export, no boundary
  refinement or manifold tracing.
- No global proof of basin geometry, measure, or completeness — the map is a
  finite deterministic sample, not a theorem.
