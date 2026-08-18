# Bifurcation analysis boundary (v2-A)

This directory specifies **deterministic parameter continuation & bifurcation
detection** — sweeping a scalar parameter through a discovered vector field,
tracking how its fixed points and their stability change, and flagging the
parameter values where a Jacobian eigenvalue crosses the imaginary axis. It is
implemented in `crates/lawsynth-bifurcation`. Like the other documents here it is
a **boundary specification** in the house style: it states what a conforming
implementation MUST do, and — crucially — what a bifurcation report is and is not
allowed to claim.

## Motivation

A discovered law frequently carries a **control parameter**: a scalar `μ` that
appears in the right-hand sides, `ẋ_i = f_i(x; μ)`. As `μ` varies, the number,
location, and stability of the equilibria can change **qualitatively** — a stable
rest state can be born or destroyed, two equilibria can collide and annihilate,
or a steady state can give way to an oscillation. These qualitative changes are
**bifurcations**, and locating them is how one turns a static list of fixed
points into a map of the system's regimes.

Linearization organizes the picture. At a hyperbolic fixed point the eigenvalues
of the Jacobian give the local behaviour; a bifurcation is exactly where a fixed
point stops being hyperbolic as `μ` passes a critical value `μ*`:

- a **single real eigenvalue** crossing zero is the signature of the
  saddle-node / transcritical / pitchfork family (a *fold* in the general sense —
  a zero-eigenvalue bifurcation);
- a **complex-conjugate pair** crossing the imaginary axis with non-zero
  imaginary part is a **Hopf** bifurcation (an oscillation is born or dies).

LawSynth is **deterministic and offline**. This stage reuses the exact analytic
Jacobian of `crates/lawsynth-jacobian`, the deterministic eigensolver of
`crates/lawsynth-koopman`, and the whole fixed-point + classification pipeline of
`crates/lawsynth-stability`. It adds only three things: an exact parameter
substitution, a nearest-coordinate branch matcher, and a bisection localizer.
Identical inputs MUST yield bit-identical output.

## What a continuation report IS

A continuation report is a **linearized, grid-sampled portrait of one field over
one parameter interval**, assembled from a **fixed grid** of stability analyses.
The contract is:

1. **A grid sweep, not a proof.** The parameter is sampled on a fixed grid over
   `[μ_min, μ_max]`. At each grid value the fixed points are those the stability
   stage reaches from its seed set inside its search box. The report describes
   what happens *on that grid*, never a claim that every equilibrium or every
   bifurcation in the interval has been found.
2. **Branches are matched, not proven connected.** Fixed points at consecutive
   grid values are stitched into branches by nearest-coordinate proximity within
   a tolerance. This is an honest reconstruction of continuity; near collisions or
   fast-moving branches it can mis-associate, and the report says so rather than
   asserting global topology.
3. **Bifurcations are eigenvalue-crossing events, not normal forms.** A detected
   bifurcation records that the dominant eigenvalue's real part changed sign
   (crossing on a persisting branch) or that a branch was born/destroyed with a
   near-zero eigenvalue (a collision fold). The report distinguishes only what
   linearization can see — real crossing (`Fold`) versus complex pair (`Hopf`) —
   and does **not** claim which specific zero-eigenvalue bifurcation occurred.
4. **A property of the expression, not of any data.** Continuation differentiates
   and evaluates whatever parameterized field it is handed; it carries no
   discovery confidence or fit residual. Those are separate, upstream concerns.

## Requirements

1. **Exact parameter substitution.** Given a field `f_i(x; μ)` as `lawsynth-expr`
   trees and a parameter value `v`, an implementation MUST produce the
   parameter-free field `f_i(x; v)` by replacing every occurrence of the parameter
   symbol with the constant `v`, structurally and exactly (no folding required,
   none forbidden). The resulting field MUST be autonomous — every remaining
   symbol a state — so the stability stage accepts it.

2. **Deterministic grid sweep.** The parameter MUST be sampled on a fixed grid of
   `steps ≥ 2` points spanning `[μ_min, μ_max]`, with the endpoints reproduced
   exactly and interior points at `μ_min + k·(μ_max − μ_min)/(steps − 1)`. The
   grid MUST be a pure function of `(μ_min, μ_max, steps)` — never random, never
   wall-clock derived. At each grid value the fixed points MUST be located and
   classified by `lawsynth-stability`; a residual stability fault MUST surface as
   a typed error carrying the offending parameter value, never be swallowed.

3. **Branch assembly by proximity.** Fixed points at consecutive grid values MUST
   be matched into branches by nearest-coordinate distance within a configurable
   tolerance, resolved by a deterministic greedy rule (smallest distance first,
   fixed tie-breaks). A branch that cannot be continued MUST end; a fixed point
   with no predecessor MUST start a new branch. Branch identifiers MUST be
   assigned in a fixed creation order.

4. **Crossing detection.** Along each branch, an implementation MUST detect where
   the **dominant eigenvalue** (greatest real part, deterministic tie-break)
   changes the side of the imaginary axis it sits on, using a configurable band
   around zero. Such a sign change MUST be reported as a bifurcation and localized
   (see 6). The event MUST be classified **Hopf** if the crossing eigenvalue has
   `|Im| >` a configurable threshold, else a real zero-eigenvalue **Fold**.

5. **Collision-fold detection.** Where a branch is born or destroyed (its fixed
   points appear/disappear between adjacent grid values) **and** the branch's
   terminal fixed point has a dominant eigenvalue with `|Re| ≤` a configurable
   fold tolerance, an implementation MUST report a **Fold**. This captures
   saddle-node collisions, where fixed points vanish and no sign change occurs on
   any persisting branch. The eigenvalue gate MUST be applied so a fixed point
   merely crossing the search-box boundary is **not** mislabelled a bifurcation.

6. **Deterministic localization.** Each critical value MUST be localized inside
   its bracketing grid interval by deterministic **bisection**: on the sign of the
   dominant eigenvalue's real part for a crossing (`BisectionOnRealPart`), or on
   fixed-point existence for a collision fold (`BisectionOnExistence`). The number
   of iterations MUST be a fixed, configured budget. Detected bifurcations that
   coincide (same kind, within a parameter and a coordinate tolerance) MUST be
   merged, so a single collision reported from several branches is one event.

7. **Determinism.** Substitution, grid generation, per-value stability, branch
   matching, crossing/fold detection, localization, and de-duplication MUST all be
   deterministic. Identical `(fields, states, parameter, Sweep, StabilityConfig)`
   inputs MUST produce a **bit-identical** `ContinuationReport`: identical grid
   values, sample reports, branch points, and bifurcation parameters/eigenvalues
   down to their `f64` bit patterns.

8. **Totality.** The state space MUST be non-empty and the parameter MUST NOT be
   one of the states (a symbol cannot be both swept and evolved). An ill-formed
   sweep (fewer than two steps, inverted range, non-finite bound, negative
   tolerance) MUST surface as a distinct typed error — never a silently truncated
   or fabricated result.

## Public API

```text
continuation(&[(Identifier, Expr)], &[Identifier], &Identifier,
             &Sweep, &StabilityConfig)
    -> Result<ContinuationReport, BifurcationError>

substitute(&Expr, &Identifier, f64) -> Expr        // exact parameter binding

Sweep::new(min, max, steps) -> Self                // + with_* builder setters
ContinuationReport { states, parameter, samples, branches, bifurcations }
ParameterSample { parameter_value, report }        // one StabilityReport per μ
Branch { id, points }                              // points: Vec<BranchPoint>
BranchPoint { parameter_value, coordinates, eigenvalues, classification }
Bifurcation { branch_id, parameter_value, kind, localization, fixed_point, eigenvalue }
BifurcationKind { Fold, Hopf }
Localization { BisectionOnRealPart, BisectionOnExistence }
ContinuationReport::to_canonical_string() -> String   // determinism fingerprint
```

`Sweep` carries the parameter range and step count, the branch-matching
tolerance, the localization iteration budget, the crossing band, the Hopf-vs-fold
imaginary threshold, the fold-acceptance eigenvalue bound, and the
bifurcation-merge tolerances. `StabilityConfig` is threaded straight through to
`lawsynth-stability` for the per-value fixed-point search.

## Honest scope & limits

- **Grid resolution bounds what is seen.** A bifurcation only registers if the
  grid straddles it: two events between adjacent grid values, or one that flips
  and flips back within a step, can be missed. A finer grid finds more, at more
  cost (`steps` × the stability cost per value); nothing here proves the
  bifurcation set is complete.
- **Branch matching by proximity can misassociate.** Near a collision, or where a
  branch moves faster per step than the match tolerance, the nearest-coordinate
  rule may join the wrong points or split one branch into several. The match
  tolerance is a modelling choice the implementation MUST expose; it cannot be
  right for every geometry at once.
- **Only the linearized family is named.** A real eigenvalue through zero is
  reported as the generic `Fold` — saddle-node, transcritical, and pitchfork are
  **not** distinguished, because telling them apart requires center-manifold or
  normal-form analysis this crate does not perform. `Hopf` likewise names the
  linear crossing, not its criticality (sub- vs supercritical).
- **Linearization is blind to global bifurcations.** Homoclinic and heteroclinic
  connections, saddle-node-of-cycles, and other global or purely-nonlinear events
  leave no fixed-point eigenvalue signature and are outside this stage entirely.
- **Everything inherits the stability stage's limits.** Fixed points are only
  those reachable from the seed set inside the search box; a bifurcation on an
  equilibrium the search never finds is never seen. The classification band and
  Newton tolerances of `StabilityConfig` apply unchanged at every parameter value.
- **Supported functions are exactly the IR's** (`+ − × ÷`, `^`, negate, `exp`,
  `log`, `sin`, `cos`), inherited from the analytic Jacobian; no derivative is
  emitted or claimed for anything outside it.

## Non-goals

- No pseudo-arclength or homotopy continuation around folds; the sweep is a plain
  parameter grid, so a branch that turns back in `μ` is seen only as a
  birth/death, not tracked around the turning point.
- No normal-form reduction, center-manifold computation, or sub/supercritical
  classification; the reported kind is the linear crossing only.
- No two-parameter continuation, codimension-two points, or bifurcation-curve
  tracing; the parameter is a single scalar.
- No limit-cycle continuation, global bifurcation detection, or basin analysis —
  only fixed-point branches and their linearized crossings.
