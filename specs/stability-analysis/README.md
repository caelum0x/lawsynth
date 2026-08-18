# Stability analysis boundary (v2-A)

This directory specifies **deterministic fixed-point & linear-stability
analysis** — locating the equilibria of a discovered vector field and
classifying each by linearization, implemented in `crates/lawsynth-stability`.
It is a **boundary specification** in the house style: it states what a
conforming implementation MUST do, and — crucially — what a stability verdict is
and is not allowed to claim.

## Motivation

A discovered dynamical law is an autonomous vector field: one right-hand side per
state, `ẋ_i = f_i(x)`, each `f_i` an expression tree in the `lawsynth-expr` IR.
The qualitative behaviour of such a system is organized around its **fixed
points** `x*` where `f(x*) = 0`, and the **local** behaviour near each is read
off the eigenvalues of the Jacobian `J = ∂f/∂x` evaluated there:

- **Equilibria** are where the modelled system can rest. Finding them is the
  first question asked of any newly discovered law.
- **Linearization** (Hartman–Grobman) says that at a *hyperbolic* fixed point —
  no eigenvalue on the imaginary axis — the nonlinear flow is locally
  topologically equivalent to its linearization, so the eigenvalue signs give an
  honest classification (stable node, saddle, spiral, …).
- **Non-hyperbolic** points (an eigenvalue with zero real part) are exactly where
  linearization is silent: a linear center may be a nonlinear spiral, a zero
  eigenvalue hides a saddle-node or transcritical bifurcation. A faithful tool
  must refuse to over-claim here.

LawSynth is **deterministic and offline**. This stage reuses the exact analytic
Jacobian of `crates/lawsynth-jacobian` and the deterministic eigensolver of
`crates/lawsynth-koopman`; the only local numerics are Newton's iteration and a
small dense linear solve. Identical inputs MUST yield bit-identical output.

## What a stability report IS

A stability report is a **local, linearized readout of the given field**, found
from a **fixed seed set**. The contract is:

1. **Fixed points are found, not proven exhaustive.** Roots are located by
   Newton's method started from a deterministic lattice over a caller-supplied
   search box (plus the origin). The report lists the roots reachable from that
   seed set inside that box — never a claim that these are *all* the equilibria.
2. **Classification is the linearization's verdict, not the nonlinear truth.**
   The class is a pure function of the Jacobian eigenvalues at the located point
   and a tolerance band. At a hyperbolic point this is a faithful local
   description; at a non-hyperbolic point it is explicitly reported as inconclusive.
3. **A property of the expression, not of any data.** The analysis differentiates
   and evaluates whatever field it is handed; it carries no discovery confidence
   or fit residual. Those are separate, upstream concerns.
4. **Honest about the search.** The report records how many seeds were tried and
   how many converged, so an empty result reads as "the search found nothing in
   the box", not "the system has no equilibria".

## Requirements

1. **Deterministic Newton from a fixed seed set.** Fixed points MUST be found by
   the multivariate Newton iteration `x_{k+1} = x_k − J(x_k)^{-1} f(x_k)`, using
   the analytic Jacobian for `J` and a deterministic linear solve (Gaussian
   elimination with partial pivoting; largest-magnitude pivot, lowest index on a
   tie). Seeds MUST be a fixed, content-independent grid over the search box plus
   the origin — never random, never wall-clock derived. The per-axis lattice and
   its Cartesian product MUST be enumerated in a fixed order.

2. **Honest non-convergence.** A seed whose iteration hits a singular Jacobian, a
   non-finite or runaway iterate, or a numeric evaluation failure MUST be
   **dropped**, never turned into a fabricated root. The report MUST state how
   many seeds converged.

3. **De-duplication and ordering.** Converged roots MUST be merged within a
   configurable per-coordinate tolerance and returned in a canonical order
   (lexicographic by coordinate, using a total float order). Roots outside the
   search box (beyond a small margin) MUST be dropped, so the box bounds the
   report.

4. **Classification rule.** From the Jacobian eigenvalues `{λ}` at a fixed point
   and a marginal-band half-width `β ≥ 0`, an implementation MUST classify by:
   - each `Re(λ)` is **negative** if `Re(λ) < −β`, **positive** if `Re(λ) > β`,
     else **marginal** (`|Re(λ)| ≤ β`);
   - **oscillation** is present if any `|Im(λ)| > β`.

   | Condition | Class |
   |---|---|
   | all negative, no oscillation | `StableNode` |
   | all negative, with oscillation | `StableSpiral` |
   | all positive, no oscillation | `UnstableNode` |
   | all positive, with oscillation | `UnstableSpiral` |
   | mixed sign, none marginal | `Saddle` |
   | all marginal, with oscillation | `Center` |
   | any marginal, otherwise | `Marginal` |

   `Center` and `Marginal` are **non-hyperbolic**: the implementation MUST mark
   them inconclusive and MUST NOT report a definitive node/spiral/saddle when an
   eigenvalue sits in the band.

5. **Reuse the deterministic eigensolver.** Eigenvalues MUST come from the
   `lawsynth-koopman` eigensolver (Householder-Hessenberg + Wilkinson-shifted
   complex QR), not a second hand-rolled solver, so the whole pipeline shares one
   audited, deterministic decomposition. Eigenvalues MUST be reported in that
   solver's canonical order.

6. **Determinism.** Seed generation, Newton, de-duplication, ordering, and
   classification MUST be deterministic. Identical `(fields, states,
   StabilityConfig)` inputs MUST produce a **bit-identical** `StabilityReport`:
   identical fixed-point coordinates (to `f64` bit patterns), classifications,
   and eigenvalue sets in identical order.

7. **Autonomy and totality.** The field MUST be autonomous: every symbol it
   references MUST be one of the states, otherwise there is no value to evaluate
   at and the implementation MUST return a typed error. Structural faults
   (duplicate state, duplicate or missing field, undifferentiable node,
   dimension mismatch between states and search box) MUST surface as distinct
   typed errors — never a silently dropped or fabricated result.

## Public API

```text
analyze_stability(&[(Identifier, Expr)], &[Identifier], &StabilityConfig)
    -> Result<StabilityReport, StabilityError>

StabilityConfig::new(search_box) -> Self          // + with_* builder setters
StabilityReport { states, fixed_points, seeds_total, seeds_converged }
FixedPoint { coordinates, eigenvalues, classification }
Classification { StableNode, StableSpiral, UnstableNode, UnstableSpiral,
                 Saddle, Center, Marginal }
StabilityReport::to_canonical_string() -> String  // determinism fingerprint
```

`StabilityConfig` carries the search box, grid resolution, Newton max-iterations
and residual tolerance, the root-merge tolerance, and the marginal band. This
crate delivers the **fixed-point + linear-stability library** only; wiring it
into continuation, bifurcation tracking, or a discovery report is downstream and
out of scope here.

## Honest scope & limits

- **Only roots reachable from the seed set in the box are found.** Newton's basin
  of attraction is local; equilibria outside the box, or inside it but not
  reachable from any seed, are missed. A finer grid or a wider box finds more, at
  more cost (`grid^n` seeds); nothing here proves a root count is complete.
- **Non-hyperbolic points cannot be resolved by linearization.** A `Center` from
  purely imaginary eigenvalues may be a nonlinear spiral; a `Marginal` zero
  eigenvalue hides a bifurcation. These verdicts are explicitly inconclusive —
  deciding them needs center-manifold or normal-form analysis this crate does not
  perform.
- **The band is a modelling choice.** A wider marginal band calls more points
  non-hyperbolic (conservative); a narrower band commits to a definitive class
  nearer the imaginary axis (riskier). The implementation MUST expose it and MUST
  NOT silently widen or narrow it to force a verdict.
- **Strong nonlinearity and stiffness stress the search.** Slowly-converging or
  clustered roots may need a tighter Newton tolerance and a larger merge radius;
  the classification is taken at the converged approximate root, so the band must
  absorb the small residual Jacobian error there.
- **Supported functions are exactly the IR's** (`+ − × ÷`, `^`, negate, `exp`,
  `log`, `sin`, `cos`), inherited from the analytic Jacobian; no derivative is
  emitted or claimed for anything outside it.

## Non-goals

- No global/exhaustive root finding, homotopy continuation, or interval
  arithmetic proof of completeness.
- No bifurcation detection, center-manifold reduction, or normal-form
  computation for non-hyperbolic points.
- No basins of attraction, Lyapunov functions, limit-cycle detection, or
  global/nonlinear stability claims — only fixed points and their linearization.
